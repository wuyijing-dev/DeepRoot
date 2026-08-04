//! Synchronous IPC stubs — Ledger Vein (0.4.x).
//!
//! Single-hart rendezvous: one pending call and one reply per endpoint.
//! Blocking recv/wakeup is handled by the scheduler (0.6.x).

use crate::cap::{CapError, CapSpace, TaskId, TaskTable};
use crate::ledger::LEDGER;
use crate::println;
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

/*
 * boot_demo - Ledger Vein worksheet under lesson-ipc
 */
pub fn boot_demo(tasks: &mut TaskTable, eps: &mut EndpointTable) {
    use crate::syscall;

    let init = tasks.spawn("init").expect("spawn init");
    let ping = tasks.spawn("ping").expect("spawn ping");

    const EP_BADGE: u64 = 0xE001;
    eps.create(init, EP_BADGE).expect("create ep");

    let (ep_slot_init, client_view) = {
        let init_cs = tasks.cspace_mut(init).expect("init cs");
        let root = init_cs
            .mint_root(rights::ALL, CapType::Untyped, CapReason::BootRoot)
            .expect("root");
        let ep = init_cs
            .mint_badged(
                root,
                rights::READ | rights::IPC | rights::GRANT,
                CapType::Endpoint,
                EP_BADGE,
                CapReason::Badge,
            )
            .expect("ep cap");
        let client = init_cs
            .derive(ep, rights::IPC, u64::MAX, CapReason::Derive)
            .expect("client derive");
        let view = init_cs.view(client).expect("view");
        (ep, view)
    };

    let ep_slot_ping = {
        let ping_cs = tasks.cspace_mut(ping).expect("ping cs");
        ping_cs
            .install_copy(
                CapType::Endpoint,
                client_view.rights,
                client_view.badge,
                CapReason::Mint,
            )
            .expect("ping ep")
    };

    println!(
        "ipc: endpoint badge={:#x} init_slot={} ping_slot={}",
        EP_BADGE, ep_slot_init, ep_slot_ping
    );

    let mut req = IpcMessage::with_label(0xC0DE);
    req.words[0] = 42;
    req.transfer_valid = 1;
    req.transfer_type = CapType::Frame as u8;
    req.transfer_rights = rights::READ | rights::WRITE;
    req.transfer_badge = 0xF00D;

    call_from_cap(tasks, eps, ping, ep_slot_ping, req).expect("ipc call/send");

    let got = {
        let init_cs = tasks.cspace_mut(init).expect("init cs");
        eps.recv(init, EP_BADGE, init_cs).expect("ipc recv")
    };
    println!(
        "ipc: init recv label={:#x} word0={} granted_slot={}",
        got.label, got.words[0], got.words[3]
    );

    let mut rep = IpcMessage::with_label(0xABCD);
    rep.words[0] = got.words[0] + 1;
    eps.reply(init, EP_BADGE, rep).expect("ipc reply");

    let reply = eps.take_reply(ping, EP_BADGE).expect("take reply");
    println!(
        "ipc: ping got reply label={:#x} word0={}",
        reply.label, reply.words[0]
    );

    let dump_rc = syscall::dispatch(
        tasks,
        eps,
        init,
        deeproot_abi::syscall::SYS_LEDGER_DUMP,
        0,
        0,
        0,
        0,
    );
    println!("ipc: SYS_LEDGER_DUMP => {}", dump_rc);

    let call_rc = syscall::dispatch(
        tasks,
        eps,
        ping,
        deeproot_abi::syscall::SYS_IPC_CALL,
        ep_slot_ping as u64,
        0x1111,
        7,
        0,
    );
    println!("ipc: SYS_IPC_CALL => {}", call_rc);
    if call_rc == 0 {
        let init_cs = tasks.cspace_mut(init).expect("init cs");
        let m = eps.recv(init, EP_BADGE, init_cs).expect("recv after syscall");
        println!(
            "ipc: post-syscall recv label={:#x} word0={}",
            m.label, m.words[0]
        );
        let mut r = IpcMessage::with_label(0x2222);
        r.words[0] = m.words[0];
        eps.reply(init, EP_BADGE, r).expect("reply2");
        let _ = eps.take_reply(ping, EP_BADGE).expect("reply2 take");
    }
}
