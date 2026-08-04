//! Synchronous IPC — call / recv / reply on endpoints (0.4.x).
//!
//! Single-hart rendezvous: one pending call and one reply per endpoint.
//! Blocking recv/wakeup is handled by the scheduler (0.6.x).

use crate::cap::{CapError, CapSpace, TaskId, TaskTable};
use crate::ledger::LEDGER;
use deeproot_abi::{rights, CapReason, CapType, IpcMessage, LedgerKind};

pub const MAX_ENDPOINTS: usize = 16;

#[derive(Debug, PartialEq, Eq)]
pub enum IpcError {
    BadCap,
    BadRights,
    NoEndpoint,
    Busy,
    Empty,
    NoReply,
    Cap(CapError),
}

impl From<CapError> for IpcError {
    fn from(e: CapError) -> Self {
        IpcError::Cap(e)
    }
}

struct Endpoint {
    live: bool,
    badge: u64,
    owner: TaskId,
    pending: Option<IpcMessage>,
    reply: Option<IpcMessage>,
    caller_task: Option<TaskId>,
}

impl Endpoint {
    const fn empty() -> Self {
        Self {
            live: false,
            badge: 0,
            owner: TaskId(0),
            pending: None,
            reply: None,
            caller_task: None,
        }
    }
}

pub struct EndpointTable {
    eps: [Endpoint; MAX_ENDPOINTS],
}

impl EndpointTable {
    pub const fn new() -> Self {
        Self {
            eps: [const { Endpoint::empty() }; MAX_ENDPOINTS],
        }
    }

    fn find_badge(&self, badge: u64) -> Option<usize> {
        self.eps.iter().position(|e| e.live && e.badge == badge)
    }

    /*
     * create - register an endpoint identified by badge
     * @owner: task that will recv/reply
     * @badge: unique badge (matches CapType::Endpoint badge)
     */
    pub fn create(&mut self, owner: TaskId, badge: u64) -> Result<(), IpcError> {
        if self.find_badge(badge).is_some() {
            return Err(IpcError::Busy);
        }
        let idx = self.eps.iter().position(|e| !e.live).ok_or(IpcError::Busy)?;
        self.eps[idx] = Endpoint {
            live: true,
            badge,
            owner,
            pending: None,
            reply: None,
            caller_task: None,
        };
        Ok(())
    }

    /*
     * send - queue a call message on the endpoint (client side)
     */
    pub fn send(&mut self, caller: TaskId, badge: u64, msg: IpcMessage) -> Result<(), IpcError> {
        let idx = self.find_badge(badge).ok_or(IpcError::NoEndpoint)?;
        let ep = &mut self.eps[idx];
        if ep.pending.is_some() {
            return Err(IpcError::Busy);
        }
        ep.pending = Some(msg);
        ep.caller_task = Some(caller);
        ep.reply = None;
        LEDGER.record(
            LedgerKind::IpcSend,
            caller.0 as u32,
            badge as u32,
            msg.label as u32,
        );
        Ok(())
    }

    /*
     * recv - server takes the pending call; installs transferred caps
     */
    pub fn recv(
        &mut self,
        owner: TaskId,
        badge: u64,
        server_cs: &mut CapSpace,
    ) -> Result<IpcMessage, IpcError> {
        let idx = self.find_badge(badge).ok_or(IpcError::NoEndpoint)?;
        let ep = &mut self.eps[idx];
        if ep.owner != owner {
            return Err(IpcError::BadRights);
        }
        let mut msg = ep.pending.take().ok_or(IpcError::Empty)?;
        if msg.transfer_valid != 0 {
            let slot = server_cs.install_copy(
                match msg.transfer_type {
                    x if x == CapType::Endpoint as u8 => CapType::Endpoint,
                    x if x == CapType::Frame as u8 => CapType::Frame,
                    _ => CapType::Untyped,
                },
                msg.transfer_rights,
                msg.transfer_badge,
                CapReason::Mint,
            )?;
            msg.words[3] = slot as u64;
        }
        LEDGER.record(
            LedgerKind::IpcRecv,
            owner.0 as u32,
            badge as u32,
            msg.label as u32,
        );
        Ok(msg)
    }

    /*
     * reply - server completes the outstanding call
     */
    pub fn reply(&mut self, owner: TaskId, badge: u64, msg: IpcMessage) -> Result<(), IpcError> {
        let idx = self.find_badge(badge).ok_or(IpcError::NoEndpoint)?;
        let ep = &mut self.eps[idx];
        if ep.owner != owner {
            return Err(IpcError::BadRights);
        }
        if ep.caller_task.is_none() {
            return Err(IpcError::NoReply);
        }
        ep.reply = Some(msg);
        LEDGER.record(
            LedgerKind::IpcSend,
            owner.0 as u32,
            badge as u32,
            msg.label as u32,
        );
        Ok(())
    }

    /*
     * take_reply - client collects the server reply
     */
    pub fn take_reply(&mut self, caller: TaskId, badge: u64) -> Result<IpcMessage, IpcError> {
        let idx = self.find_badge(badge).ok_or(IpcError::NoEndpoint)?;
        let ep = &mut self.eps[idx];
        if ep.caller_task != Some(caller) {
            return Err(IpcError::BadRights);
        }
        let msg = ep.reply.take().ok_or(IpcError::Empty)?;
        ep.caller_task = None;
        LEDGER.record(
            LedgerKind::IpcRecv,
            caller.0 as u32,
            badge as u32,
            msg.label as u32,
        );
        Ok(msg)
    }
}

impl Default for EndpointTable {
    fn default() -> Self {
        Self::new()
    }
}

/*
 * call_from_cap - client send using an Endpoint capability slot
 */
pub fn call_from_cap(
    tasks: &TaskTable,
    eps: &mut EndpointTable,
    caller: TaskId,
    ep_slot: usize,
    msg: IpcMessage,
) -> Result<(), IpcError> {
    let cs = tasks.cspace(caller).ok_or(IpcError::BadCap)?;
    let slot = cs.get(ep_slot).ok_or(IpcError::BadCap)?;
    if slot.cap_type != CapType::Endpoint {
        return Err(IpcError::BadCap);
    }
    if slot.rights & rights::IPC == 0 {
        return Err(IpcError::BadRights);
    }
    eps.send(caller, slot.badge, msg)
}
