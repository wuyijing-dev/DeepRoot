//! Schedule Canopy (0.6.x) — TCB, RR, timer preemption, block/wakeup, idle.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::cap::{TaskId, TaskTable};
use crate::ipc::EndpointTable;
use crate::mm::frame;
use crate::mm::layout::{PhysAddr, PAGE_SIZE};
use crate::mm::sv39;
use crate::println;
use crate::timer;

pub const MAX_UTASKS: usize = 8;
pub const USER_STACK_PAGES: usize = 4;
pub const KSTACK_PAGES: usize = 2;

const IDLE_VA: usize = 0x1300_0000;
const IDLE_STACK: usize = 0x1310_0000;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    pub x: [usize; 32],
    pub sepc: usize,
    pub sstatus: usize,
}

impl TrapFrame {
    pub const fn zero() -> Self {
        Self {
            x: [0; 32],
            sepc: 0,
            sstatus: 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskState {
    Empty,
    Ready,
    Running,
    Blocked,
    Zombie,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockReason {
    None,
    IpcRecv { badge: u64 },
    IpcCall { badge: u64 },
}

/// Thread Control Block.
pub struct UserTask {
    pub state: TaskState,
    pub name: &'static str,
    pub cap_task: TaskId,
    pub tf: TrapFrame,
    pub stack_top: usize,
    /// Dedicated kernel stack top (single-hart traps still use boot stack;
    /// field reserved for multi-hart / nested IRQ work — 0.6.0 / 0.6.5).
    pub kstack_top: usize,
    pub block: BlockReason,
    pub is_idle: bool,
}

impl UserTask {
    const fn empty() -> Self {
        Self {
            state: TaskState::Empty,
            name: "",
            cap_task: TaskId(0),
            tf: TrapFrame::zero(),
            stack_top: 0,
            kstack_top: 0,
            block: BlockReason::None,
            is_idle: false,
        }
    }
}

struct SchedInner {
    tasks: [UserTask; MAX_UTASKS],
    current: usize,
    idle_id: Option<usize>,
    preempt_count: u64,
}

struct Sched(UnsafeCell<SchedInner>);
unsafe impl Sync for Sched {}

static SCHED: Sched = Sched(UnsafeCell::new(SchedInner {
    tasks: [const { UserTask::empty() }; MAX_UTASKS],
    current: 0,
    idle_id: None,
    preempt_count: 0,
}));

static CURRENT_TF: AtomicUsize = AtomicUsize::new(0);

fn inner() -> &'static mut SchedInner {
    unsafe { &mut *SCHED.0.get() }
}

pub fn current_tf_ptr() -> usize {
    CURRENT_TF.load(Ordering::Relaxed)
}

pub fn set_current_tf(tf: *mut TrapFrame) {
    CURRENT_TF.store(tf as usize, Ordering::Relaxed);
}

/*
 * set_syscall_return - write a0 for the task that issued the ecall
 *
 * Parked IPC callers (IpcCall) keep a0 until complete_call delivers the reply.
 */
pub fn set_syscall_return(task_id: usize, ret: isize) {
    let s = inner();
    if task_id < MAX_UTASKS && s.tasks[task_id].state != TaskState::Empty {
        if matches!(s.tasks[task_id].block, BlockReason::IpcCall { .. }) {
            return;
        }
        s.tasks[task_id].tf.x[10] = ret as usize;
    }
}

pub fn find_sched_id(cap: TaskId) -> Option<usize> {
    let s = inner();
    s.tasks
        .iter()
        .position(|t| t.state != TaskState::Empty && t.cap_task == cap)
}

/*
 * complete_call - deliver reply label to a blocked IPC caller and Ready it
 */
pub fn complete_call(sched_id: usize, ret: isize) {
    let s = inner();
    if sched_id >= MAX_UTASKS {
        return;
    }
    s.tasks[sched_id].tf.x[10] = ret as usize;
    if s.tasks[sched_id].state == TaskState::Blocked {
        s.tasks[sched_id].state = TaskState::Ready;
        s.tasks[sched_id].block = BlockReason::None;
    }
}

/*
 * abort_ipc_waiters - wake recv/call waiters on @badge with an error in a0
 */
pub fn abort_ipc_waiters(badge: u64, err: isize) {
    let s = inner();
    for t in s.tasks.iter_mut() {
        if t.state != TaskState::Blocked {
            continue;
        }
        let match_b = match t.block {
            BlockReason::IpcRecv { badge: b } | BlockReason::IpcCall { badge: b } => b == badge,
            BlockReason::None => false,
        };
        if match_b {
            t.tf.x[10] = err as usize;
            t.state = TaskState::Ready;
            t.block = BlockReason::None;
        }
    }
}

pub fn current_id() -> usize {
    inner().current
}

pub fn current_cap_task() -> TaskId {
    let s = inner();
    s.tasks[s.current].cap_task
}

fn alloc_kstack() -> Option<usize> {
    let mut first = None;
    let mut last = PhysAddr::new(0);
    for i in 0..KSTACK_PAGES {
        let f = frame::alloc()?;
        if i == 0 {
            first = Some(f.as_usize());
        } else {
            assert_eq!(f.as_usize(), last.as_usize() + PAGE_SIZE);
        }
        last = f;
    }
    Some(first.unwrap() + KSTACK_PAGES * PAGE_SIZE)
}

/*
 * spawn - create a U-mode task around a loaded ELF entry
 */
pub fn spawn(
    name: &'static str,
    entry: usize,
    stack_va_base: usize,
    cap_task: TaskId,
) -> Option<usize> {
    spawn_inner(name, entry, stack_va_base, cap_task, false)
}

fn spawn_inner(
    name: &'static str,
    entry: usize,
    stack_va_base: usize,
    cap_task: TaskId,
    is_idle: bool,
) -> Option<usize> {
    let s = inner();
    let idx = s.tasks.iter().position(|t| t.state == TaskState::Empty)?;
    for i in 0..USER_STACK_PAGES {
        let frame = frame::alloc()?;
        sv39::map_user(stack_va_base + i * PAGE_SIZE, frame, false, true);
    }
    let stack_top = stack_va_base + USER_STACK_PAGES * PAGE_SIZE;
    let kstack_top = alloc_kstack().unwrap_or(0);

    const SSTATUS_SPIE: usize = 1 << 5;
    const SSTATUS_SUM: usize = 1 << 18;

    let mut tf = TrapFrame::zero();
    tf.sepc = entry;
    tf.sstatus = SSTATUS_SPIE | SSTATUS_SUM;
    tf.x[2] = stack_top;

    s.tasks[idx] = UserTask {
        state: TaskState::Ready,
        name,
        cap_task,
        tf,
        stack_top,
        kstack_top,
        block: BlockReason::None,
        is_idle,
    };
    if is_idle {
        s.idle_id = Some(idx);
    }
    println!(
        "sched: spawn {} id={} entry={:#x} sp={:#x} kstack={:#x} idle={}",
        name, idx, entry, stack_top, kstack_top, is_idle
    );
    Some(idx)
}

/*
 * spawn_idle - map a tiny U-mode yield loop and park it as the idle thread
 */
pub fn spawn_idle(cap_task: TaskId) -> Option<usize> {
    let page = frame::alloc()?;
    /* Position-independent loop: li a7, SYS_YIELD; ecall; j start */
    let code: [u32; 3] = [
        0x0080_0893, /* addi a7, x0, 8 */
        0x0000_0073, /* ecall */
        0xff9f_f06f, /* jal x0, -8 → addi */
    ];
    unsafe {
        let dst = page.as_usize() as *mut u32;
        for (i, w) in code.iter().enumerate() {
            dst.add(i).write(*w);
        }
    }
    sv39::map_user(IDLE_VA, page, true, false);
    spawn_inner("idle", IDLE_VA, IDLE_STACK, cap_task, true)
}

pub fn mark_zombie(code: usize) {
    let s = inner();
    let id = s.current;
    if s.tasks[id].is_idle {
        return;
    }
    s.tasks[id].state = TaskState::Zombie;
    s.tasks[id].block = BlockReason::None;
    println!("sched: {} exited code={}", s.tasks[id].name, code);
}

/*
 * kill_current - mark current U-task Zombie after a fatal trap
 */
pub fn kill_current(reason: &'static str) {
    let s = inner();
    let id = s.current;
    if s.tasks[id].is_idle {
        return;
    }
    println!("sched: {} killed ({})", s.tasks[id].name, reason);
    s.tasks[id].state = TaskState::Zombie;
    s.tasks[id].block = BlockReason::None;
}

/*
 * block_current_ipc - mark current task Blocked waiting to recv on @badge
 */
pub fn block_current_ipc(badge: u64) {
    let s = inner();
    let id = s.current;
    if s.tasks[id].is_idle {
        return;
    }
    s.tasks[id].state = TaskState::Blocked;
    s.tasks[id].block = BlockReason::IpcRecv { badge };
}

/*
 * block_current_call - mark current task Blocked waiting for a reply on @badge
 */
pub fn block_current_call(badge: u64) {
    let s = inner();
    let id = s.current;
    if s.tasks[id].is_idle {
        return;
    }
    s.tasks[id].state = TaskState::Blocked;
    s.tasks[id].block = BlockReason::IpcCall { badge };
}

/*
 * wakeup_ipc - Ready any task blocked in recv on @badge
 */
pub fn wakeup_ipc(badge: u64) {
    let s = inner();
    for t in s.tasks.iter_mut() {
        if t.state == TaskState::Blocked {
            if let BlockReason::IpcRecv { badge: b } = t.block {
                if b == badge {
                    t.state = TaskState::Ready;
                    t.block = BlockReason::None;
                }
            }
        }
    }
}

fn pick_next(from: usize) -> Option<usize> {
    let s = inner();
    for off in 1..=MAX_UTASKS {
        let i = (from + off) % MAX_UTASKS;
        if s.tasks[i].state == TaskState::Ready && !s.tasks[i].is_idle {
            return Some(i);
        }
    }
    /* Idle only when nothing else is runnable. */
    if let Some(idle) = s.idle_id {
        s.tasks[idle].state = TaskState::Ready;
        s.tasks[idle].block = BlockReason::None;
        return Some(idle);
    }
    None
}

/*
 * yield_now - round-robin to next Ready task (0.6.1)
 */
pub fn yield_now() -> bool {
    let s = inner();
    let cur = s.current;
    if let Some(next) = pick_next(cur) {
        if s.tasks[cur].state == TaskState::Running {
            if !s.tasks[cur].is_idle {
                s.tasks[cur].state = TaskState::Ready;
            } else {
                s.tasks[cur].state = TaskState::Ready;
            }
        }
        s.current = next;
        s.tasks[next].state = TaskState::Running;
        set_current_tf(&mut s.tasks[next].tf as *mut TrapFrame);
        return true;
    }
    false
}

/*
 * preempt - timer quantum expired (0.6.2 / 0.6.4)
 */
pub fn preempt() {
    let s = inner();
    s.preempt_count = s.preempt_count.wrapping_add(1);
    if s.preempt_count == 1 || s.preempt_count % 50 == 0 {
        println!(
            "sched: preempt tick={} hart={} current={}",
            timer::ticks(),
            timer::hart_id(),
            s.tasks[s.current].name
        );
    }
    let _ = yield_now();
}

pub fn enter_first(id: usize) -> ! {
    let s = inner();
    s.current = id;
    s.tasks[id].state = TaskState::Running;
    set_current_tf(&mut s.tasks[id].tf as *mut TrapFrame);
    timer::note_preempt_ready();
    restore_user();
}

pub fn restore_user() -> ! {
    let s = inner();
    let id = s.current;
    let sepc = s.tasks[id].tf.sepc;
    let ra = s.tasks[id].tf.x[1];
    let sp = s.tasks[id].tf.x[2];
    if sepc == 0 || sepc < 0x1000_0000 || s.tasks[id].state == TaskState::Empty {
        println!(
            "sched: BUG restore id={} name={} state={:?} sepc={:#x} ra={:#x} sp={:#x}",
            id, s.tasks[id].name, s.tasks[id].state, sepc, ra, sp
        );
        loop {
            crate::sbi::hart_suspend_idle();
        }
    }
    set_current_tf(&mut s.tasks[id].tf as *mut TrapFrame);
    let tf = current_tf_ptr();
    unsafe {
        core::arch::asm!(
            "mv t6, {tf}",
            "ld t1, 32*8(t6)",
            "csrw sepc, t1",
            "ld t1, 33*8(t6)",
            "csrw sstatus, t1",
            "csrw sscratch, t6",
            "ld ra, 1*8(t6)",
            "ld sp, 2*8(t6)",
            "ld gp, 3*8(t6)",
            "ld tp, 4*8(t6)",
            "ld t0, 5*8(t6)",
            "ld t1, 6*8(t6)",
            "ld t2, 7*8(t6)",
            "ld s0, 8*8(t6)",
            "ld s1, 9*8(t6)",
            "ld a0, 10*8(t6)",
            "ld a1, 11*8(t6)",
            "ld a2, 12*8(t6)",
            "ld a3, 13*8(t6)",
            "ld a4, 14*8(t6)",
            "ld a5, 15*8(t6)",
            "ld a6, 16*8(t6)",
            "ld a7, 17*8(t6)",
            "ld s2, 18*8(t6)",
            "ld s3, 19*8(t6)",
            "ld s4, 20*8(t6)",
            "ld s5, 21*8(t6)",
            "ld s6, 22*8(t6)",
            "ld s7, 23*8(t6)",
            "ld s8, 24*8(t6)",
            "ld s9, 25*8(t6)",
            "ld s10, 26*8(t6)",
            "ld s11, 27*8(t6)",
            "ld t3, 28*8(t6)",
            "ld t4, 29*8(t6)",
            "ld t5, 30*8(t6)",
            "ld t6, 31*8(t6)",
            "sret",
            tf = in(reg) tf,
            options(noreturn),
        );
    }
}

pub fn handle_syscall(
    tasks: &mut TaskTable,
    eps: &mut EndpointTable,
    nr: usize,
    a0: u64,
    a1: u64,
    a2: u64,
    a3: u64,
) -> isize {
    use deeproot_abi::syscall::*;
    use deeproot_abi::{CapReason, CapType, LedgerKind};
    let current = current_cap_task();
    match nr {
        SYS_YIELD => {
            let s = inner();
            let cur = s.current;
            if s.tasks[cur].is_idle {
                let any_ready = s.tasks.iter().enumerate().any(|(i, t)| {
                    i != cur && t.state == TaskState::Ready
                });
                if !any_ready {
                    unsafe {
                        core::arch::asm!("wfi", options(nomem, nostack));
                    }
                    let _ = timer::on_interrupt();
                }
            }
            let _ = yield_now();
            0
        }
        SYS_EXIT => {
            mark_zombie(a0 as usize);
            if !yield_now() {
                println!("sched: no runnable tasks — hanging");
                loop {
                    crate::sbi::hart_suspend_idle();
                }
            }
            0
        }
        SYS_DEBUG_WRITE => {
            let ptr = a0 as usize;
            let len = a1 as usize;
            if len > 4096 {
                return ERR_GENERIC;
            }
            let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
            if let Ok(s) = core::str::from_utf8(slice) {
                crate::console::_print(core::format_args!("{}", s));
            }
            len as isize
        }
        SYS_LEDGER_DUMP => {
            crate::ledger::LEDGER.dump_to_console();
            0
        }
        SYS_CAP_MINT => {
            let cs = match tasks.cspace_mut(current) {
                Some(c) => c,
                None => return ERR_GENERIC,
            };
            let cap_ty = match a2 as u8 {
                x if x == CapType::Untyped as u8 => CapType::Untyped,
                x if x == CapType::Endpoint as u8 => CapType::Endpoint,
                x if x == CapType::Frame as u8 => CapType::Frame,
                x if x == CapType::CNode as u8 => CapType::CNode,
                _ => return ERR_GENERIC,
            };
            match cs.mint_badged(a0 as usize, a1 as u32, cap_ty, a3, CapReason::Mint) {
                Ok(slot) => {
                    crate::ledger::LEDGER.record(
                        LedgerKind::CapMint,
                        a0 as u32,
                        slot as u32,
                        a1 as u32,
                    );
                    slot as isize
                }
                Err(_) => ERR_GENERIC,
            }
        }
        SYS_CAP_DERIVE => {
            let cs = match tasks.cspace_mut(current) {
                Some(c) => c,
                None => return ERR_GENERIC,
            };
            match cs.derive(a0 as usize, a1 as u32, a2, CapReason::Derive) {
                Ok(slot) => {
                    crate::ledger::LEDGER.record(
                        LedgerKind::CapDerive,
                        a0 as u32,
                        slot as u32,
                        a1 as u32,
                    );
                    slot as isize
                }
                Err(_) => ERR_GENERIC,
            }
        }
        SYS_CAP_REVOKE => {
            let slot = a0 as usize;
            let badge = tasks
                .cspace(current)
                .and_then(|cs| cs.get(slot))
                .filter(|s| s.cap_type == CapType::Endpoint)
                .map(|s| s.badge);
            let cs = match tasks.cspace_mut(current) {
                Some(c) => c,
                None => return ERR_GENERIC,
            };
            match cs.revoke(slot) {
                Ok(n) => {
                    if let Some(b) = badge {
                        eps.clear_badge(b);
                        abort_ipc_waiters(b, ERR_GENERIC);
                    }
                    crate::ledger::LEDGER.record(
                        LedgerKind::CapRevoke,
                        slot as u32,
                        n as u32,
                        0,
                    );
                    n as isize
                }
                Err(_) => ERR_GENERIC,
            }
        }
        SYS_IPC_CALL => {
            let mut msg = deeproot_abi::IpcMessage::with_label(a1);
            msg.words[0] = a2;
            let badge = match tasks.cspace(current).and_then(|cs| cs.get(a0 as usize)) {
                Some(s) if s.cap_type == CapType::Endpoint => s.badge,
                _ => return ERR_GENERIC,
            };
            match crate::ipc::call_from_cap(tasks, eps, current, a0 as usize, msg) {
                Ok(()) => {
                    wakeup_ipc(badge);
                    match eps.take_reply(current, badge) {
                        Ok(m) => m.label as isize,
                        Err(crate::ipc::IpcError::Empty) => {
                            block_current_call(badge);
                            let _ = yield_now();
                            0
                        }
                        Err(_) => ERR_GENERIC,
                    }
                }
                Err(_) => ERR_GENERIC,
            }
        }
        SYS_IPC_RECV => {
            let badge = a0;
            let cs = match tasks.cspace_mut(current) {
                Some(c) => c,
                None => return ERR_GENERIC,
            };
            match eps.recv(current, badge, cs) {
                Ok(m) => m.label as isize,
                Err(crate::ipc::IpcError::Empty) => {
                    block_current_ipc(badge);
                    let _ = yield_now();
                    ERR_AGAIN
                }
                Err(_) => ERR_GENERIC,
            }
        }
        SYS_IPC_REPLY => {
            let badge = a0;
            let mut msg = deeproot_abi::IpcMessage::with_label(a1);
            msg.words[0] = a2;
            let caller = match eps.caller_of(badge) {
                Some(c) => c,
                None => return ERR_GENERIC,
            };
            match eps.reply(current, badge, msg) {
                Ok(()) => match eps.take_reply(caller, badge) {
                    Ok(m) => {
                        if let Some(sid) = find_sched_id(caller) {
                            complete_call(sid, m.label as isize);
                        }
                        0
                    }
                    Err(_) => ERR_GENERIC,
                },
                Err(_) => ERR_GENERIC,
            }
        }
        _ => ERR_NOSYS,
    }
}
