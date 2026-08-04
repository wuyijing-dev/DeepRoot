//! Schedule Canopy (0.6.x) + SMP (1.7) — per-hart RQ, locks, IPI wake.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::cap::{TaskId, TaskTable};
use crate::ipc::EndpointTable;
use crate::mm::aspace::AddrSpace;
use crate::mm::frame;
use crate::mm::layout::{PhysAddr, PAGE_SIZE};
use crate::mm::sv39;
use crate::println;
use crate::smp::{self, MAX_HARTS};
use crate::sync::SpinLock;
use crate::timer;

pub const MAX_UTASKS: usize = 12;
pub const USER_STACK_PAGES: usize = 4;
pub const KSTACK_PAGES: usize = 2;

static SCHED_LOCK: SpinLock = SpinLock::new();
static NEXT_HOME: AtomicUsize = AtomicUsize::new(0);

const IDLE_VA: usize = 0x1300_0000;
const IDLE_STACK: usize = 0x1310_0000;
/// Dynamic spawn stack bases: 0x15000000 + id * 0x01000000
const SPAWN_STACK_BASE: usize = 0x1500_0000;
const SPAWN_STACK_STRIDE: usize = 0x0100_0000;

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
    #[allow(dead_code)]
    pub stack_top: usize,
    #[allow(dead_code)]
    pub kstack_top: usize,
    /// Sv39 root physical address (0 = unset).
    pub root_pa: usize,
    pub block: BlockReason,
    pub is_idle: bool,
    /// Home runqueue hart (1.7 per-hart RQ).
    pub home_hart: usize,
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
            root_pa: 0,
            block: BlockReason::None,
            is_idle: false,
            home_hart: 0,
        }
    }

    fn activate_as(&self) {
        if self.root_pa != 0 {
            sv39::activate(PhysAddr::new(self.root_pa));
        }
    }
}

struct SchedInner {
    tasks: [UserTask; MAX_UTASKS],
    /// Per-hart current task index.
    current: [usize; MAX_HARTS],
    /// Per-hart idle task index.
    idle_id: [Option<usize>; MAX_HARTS],
    preempt_count: u64,
}

struct Sched(UnsafeCell<SchedInner>);
unsafe impl Sync for Sched {}

static SCHED: Sched = Sched(UnsafeCell::new(SchedInner {
    tasks: [const { UserTask::empty() }; MAX_UTASKS],
    current: [0; MAX_HARTS],
    idle_id: [None; MAX_HARTS],
    preempt_count: 0,
}));

static CURRENT_TF: [AtomicUsize; MAX_HARTS] = [const { AtomicUsize::new(0) }; MAX_HARTS];

fn inner() -> &'static mut SchedInner {
    unsafe { &mut *SCHED.0.get() }
}

fn alloc_home_hart() -> usize {
    let n = smp::hart_count().max(1);
    NEXT_HOME.fetch_add(1, Ordering::Relaxed) % n
}

pub fn current_tf_ptr() -> usize {
    let h = smp::hart_id().min(MAX_HARTS - 1);
    CURRENT_TF[h].load(Ordering::Relaxed)
}

pub fn set_current_tf(tf: *mut TrapFrame) {
    let h = smp::hart_id().min(MAX_HARTS - 1);
    CURRENT_TF[h].store(tf as usize, Ordering::Relaxed);
}

/*
 * set_syscall_return - write a0 for the task that issued the ecall
 *
 * Parked IPC callers (IpcCall) keep a0 until complete_call delivers the reply.
 */
pub fn set_syscall_return(task_id: usize, ret: isize) {
    let _g = SCHED_LOCK.lock();
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
    let mut wake_hart = None;
    {
        let _g = SCHED_LOCK.lock();
        let s = inner();
        if sched_id >= MAX_UTASKS {
            return;
        }
        s.tasks[sched_id].tf.x[10] = ret as usize;
        if s.tasks[sched_id].state == TaskState::Blocked {
            s.tasks[sched_id].state = TaskState::Ready;
            s.tasks[sched_id].block = BlockReason::None;
            wake_hart = Some(s.tasks[sched_id].home_hart);
        }
    }
    if let Some(h) = wake_hart {
        smp::ipi_wake(h);
    }
}

/*
 * abort_ipc_waiters - wake recv/call waiters on @badge with an error in a0
 */
pub fn abort_ipc_waiters(badge: u64, err: isize) {
    let mut wake = [false; MAX_HARTS];
    {
        let _g = SCHED_LOCK.lock();
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
                if t.home_hart < MAX_HARTS {
                    wake[t.home_hart] = true;
                }
            }
        }
    }
    for h in 0..MAX_HARTS {
        if wake[h] {
            smp::ipi_wake(h);
        }
    }
}

pub fn current_id() -> usize {
    let h = smp::hart_id().min(MAX_HARTS - 1);
    inner().current[h]
}

pub fn current_cap_task() -> TaskId {
    let s = inner();
    let h = smp::hart_id().min(MAX_HARTS - 1);
    s.tasks[s.current[h]].cap_task
}

pub fn idle_id_for(hart: usize) -> Option<usize> {
    if hart >= MAX_HARTS {
        return None;
    }
    inner().idle_id[hart]
}

pub fn set_task_home(id: usize, hart: usize) {
    let _g = SCHED_LOCK.lock();
    let s = inner();
    if id < MAX_UTASKS && s.tasks[id].state != TaskState::Empty {
        s.tasks[id].home_hart = hart.min(MAX_HARTS - 1);
    }
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
 * spawn_as - create a U-mode task in @aspace (ELF already mapped there)
 */
pub fn spawn_as(
    name: &'static str,
    entry: usize,
    stack_va_base: usize,
    cap_task: TaskId,
    aspace: AddrSpace,
) -> Option<usize> {
    spawn_inner(name, entry, stack_va_base, cap_task, false, aspace)
}

fn spawn_inner(
    name: &'static str,
    entry: usize,
    stack_va_base: usize,
    cap_task: TaskId,
    is_idle: bool,
    aspace: AddrSpace,
) -> Option<usize> {
    let s = inner();
    let idx = s.tasks.iter().position(|t| t.state == TaskState::Empty)?;
    for i in 0..USER_STACK_PAGES {
        let frame = frame::alloc()?;
        aspace.map_user(stack_va_base + i * PAGE_SIZE, frame, false, true);
    }
    let stack_top = stack_va_base + USER_STACK_PAGES * PAGE_SIZE;
    let kstack_top = alloc_kstack().unwrap_or(0);
    let root_pa = aspace.root_pa.as_usize();

    const SSTATUS_SPIE: usize = 1 << 5;
    const SSTATUS_SUM: usize = 1 << 18;

    let mut tf = TrapFrame::zero();
    tf.sepc = entry;
    tf.sstatus = SSTATUS_SPIE | SSTATUS_SUM;
    tf.x[2] = stack_top;

    let home = if is_idle {
        /* Caller must set home via spawn_idle_on before use; default 0. */
        0
    } else {
        alloc_home_hart()
    };
    s.tasks[idx] = UserTask {
        state: TaskState::Ready,
        name,
        cap_task,
        tf,
        stack_top,
        kstack_top,
        root_pa,
        block: BlockReason::None,
        is_idle,
        home_hart: home,
    };
    /* Stay quiet on success — shell/fs exec should not flood the serial. */
    Some(idx)
}

/*
 * spawn_idle_on - per-hart U-mode yield loop (unique stack VA per hart)
 */
pub fn spawn_idle_on(cap_task: TaskId, hart: usize) -> Option<usize> {
    let hart = hart.min(MAX_HARTS - 1);
    let aspace = AddrSpace::create()?;
    let page = frame::alloc()?;
    let code: [u32; 3] = [
        0x0080_0893, /* addi a7, x0, 8 */
        0x0000_0073, /* ecall */
        0xff9f_f06f, /* jal x0, -8 */
    ];
    unsafe {
        let dst = page.as_usize() as *mut u32;
        for (i, w) in code.iter().enumerate() {
            dst.add(i).write(*w);
        }
    }
    /* Distinct code/stack VAs so each idle has its own pages. */
    let code_va = IDLE_VA + hart * 0x0010_0000;
    let stack_va = IDLE_STACK + hart * 0x0010_0000;
    aspace.map_user(code_va, page, true, false);
    let id = spawn_inner("idle", code_va, stack_va, cap_task, true, aspace)?;
    let _g = SCHED_LOCK.lock();
    let s = inner();
    s.tasks[id].home_hart = hart;
    s.idle_id[hart] = Some(id);
    Some(id)
}

/*
 * spawn_idle - boot-hart idle (compat wrapper)
 */
pub fn spawn_idle(cap_task: TaskId) -> Option<usize> {
    spawn_idle_on(cap_task, 0)
}

/*
 * spawn_elf_bytes - runtime spawn from an ELF image (1.1 SYS_SPAWN)
 */
pub fn spawn_elf_bytes(
    name: &'static str,
    bytes: &[u8],
    stack_va_base: usize,
    cap_task: TaskId,
) -> Option<usize> {
    let aspace = AddrSpace::create()?;
    let loaded = crate::elf::load_into(&aspace, name, bytes)?;
    spawn_inner(name, loaded.entry, stack_va_base, cap_task, false, aspace)
}

pub fn next_spawn_stack_base(sched_id: usize) -> usize {
    SPAWN_STACK_BASE + sched_id * SPAWN_STACK_STRIDE
}

pub fn mark_zombie(_code: usize) {
    let _g = SCHED_LOCK.lock();
    let s = inner();
    let h = smp::hart_id().min(MAX_HARTS - 1);
    let id = s.current[h];
    if s.tasks[id].is_idle {
        return;
    }
    s.tasks[id].state = TaskState::Zombie;
    s.tasks[id].block = BlockReason::None;
}

/*
 * kill_current - mark current U-task Zombie after a fatal trap
 */
pub fn kill_current(reason: &'static str) {
    let name;
    {
        let _g = SCHED_LOCK.lock();
        let s = inner();
        let h = smp::hart_id().min(MAX_HARTS - 1);
        let id = s.current[h];
        if s.tasks[id].is_idle {
            return;
        }
        name = s.tasks[id].name;
        s.tasks[id].state = TaskState::Zombie;
        s.tasks[id].block = BlockReason::None;
    }
    println!("sched: {} killed ({})", name, reason);
}

/*
 * block_current_ipc - mark current task Blocked waiting to recv on @badge
 */
pub fn block_current_ipc(badge: u64) {
    let _g = SCHED_LOCK.lock();
    let s = inner();
    let h = smp::hart_id().min(MAX_HARTS - 1);
    let id = s.current[h];
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
    let _g = SCHED_LOCK.lock();
    let s = inner();
    let h = smp::hart_id().min(MAX_HARTS - 1);
    let id = s.current[h];
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
    let mut wake = [false; MAX_HARTS];
    {
        let _g = SCHED_LOCK.lock();
        let s = inner();
        for t in s.tasks.iter_mut() {
            if t.state == TaskState::Blocked {
                if let BlockReason::IpcRecv { badge: b } = t.block {
                    if b == badge {
                        t.state = TaskState::Ready;
                        t.block = BlockReason::None;
                        if t.home_hart < MAX_HARTS {
                            wake[t.home_hart] = true;
                        }
                    }
                }
            }
        }
    }
    for h in 0..MAX_HARTS {
        if wake[h] {
            smp::ipi_wake(h);
        }
    }
}

fn pick_next(from: usize, hart: usize) -> Option<usize> {
    let s = inner();
    for off in 1..=MAX_UTASKS {
        let i = (from + off) % MAX_UTASKS;
        let t = &s.tasks[i];
        if t.state == TaskState::Ready && !t.is_idle && t.home_hart == hart {
            return Some(i);
        }
    }
    /* Idle only when nothing else is runnable on this hart. */
    if let Some(idle) = s.idle_id[hart] {
        s.tasks[idle].state = TaskState::Ready;
        s.tasks[idle].block = BlockReason::None;
        return Some(idle);
    }
    None
}

/*
 * yield_now - round-robin to next Ready task on this hart (0.6.1 / 1.7)
 */
pub fn yield_now() -> bool {
    let _g = SCHED_LOCK.lock();
    let s = inner();
    let h = smp::hart_id().min(MAX_HARTS - 1);
    let cur = s.current[h];
    if let Some(next) = pick_next(cur, h) {
        if s.tasks[cur].state == TaskState::Running {
            s.tasks[cur].state = TaskState::Ready;
        }
        s.current[h] = next;
        s.tasks[next].state = TaskState::Running;
        set_current_tf(&mut s.tasks[next].tf as *mut TrapFrame);
        s.tasks[next].activate_as();
        return true;
    }
    false
}

/*
 * preempt - timer quantum expired (0.6.2 / 0.6.4)
 *
 * Stay quiet on the console: tick spam fights the interactive shell's
 * serial input/echo. Count is kept for later ledger / debug dumps.
 */
pub fn preempt() {
    {
        let _g = SCHED_LOCK.lock();
        let s = inner();
        s.preempt_count = s.preempt_count.wrapping_add(1);
    }
    let _ = yield_now();
}

pub fn enter_first(id: usize) -> ! {
    {
        let _g = SCHED_LOCK.lock();
        let s = inner();
        let h = smp::hart_id().min(MAX_HARTS - 1);
        s.current[h] = id;
        s.tasks[id].home_hart = h;
        s.tasks[id].state = TaskState::Running;
        set_current_tf(&mut s.tasks[id].tf as *mut TrapFrame);
        s.tasks[id].activate_as();
    }
    if smp::hart_id() == smp::boot_hart() {
        timer::note_preempt_ready();
    }
    restore_user();
}

pub fn restore_user() -> ! {
    let s = inner();
    let h = smp::hart_id().min(MAX_HARTS - 1);
    let id = s.current[h];
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
    s.tasks[id].activate_as();
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
            /* Keep kernel hart id in `tp` (x4). Userspace TLS is not used yet;
             * restoring TF.tp would zero it and break per-hart trap stacks. */
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

/*
 * syscall_yield - SYS_YIELD without TrapCtx lock (safe to WFI on SMP)
 */
pub fn syscall_yield() -> isize {
    let h = smp::hart_id().min(MAX_HARTS - 1);
    let idle_wait = {
        let _g = SCHED_LOCK.lock();
        let s = inner();
        let cur = s.current[h];
        if s.tasks[cur].is_idle {
            let any_ready = s.tasks.iter().enumerate().any(|(i, t)| {
                i != cur
                    && t.state == TaskState::Ready
                    && !t.is_idle
                    && t.home_hart == h
            });
            !any_ready
        } else {
            false
        }
    };
    if idle_wait {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
        crate::sbi::clear_ssip();
        let mut sip: usize;
        unsafe {
            core::arch::asm!("csrr {}, sip", out(reg) sip, options(nostack));
        }
        if sip & (1 << 5) != 0 {
            let _ = timer::on_interrupt();
        }
    }
    let _ = yield_now();
    0
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
        SYS_YIELD => syscall_yield(),
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
            if len > 8192 {
                return ERR_GENERIC;
            }
            let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
            crate::console::write_bytes(slice);
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
        SYS_SPAWN => {
            let blob = a0 as usize;
            let bytes: &[u8] = match blob {
                0 => crate::servers::HELLO_ELF,
                _ => return ERR_GENERIC,
            };
            let cap = match tasks.spawn("spawned") {
                Some(t) => t,
                None => return ERR_GENERIC,
            };
            /* Reserve sched slot id for stack placement. */
            let s = inner();
            let slot = match s.tasks.iter().position(|t| t.state == TaskState::Empty) {
                Some(i) => i,
                None => return ERR_GENERIC,
            };
            let stack = next_spawn_stack_base(slot);
            match spawn_elf_bytes("hello", bytes, stack, cap) {
                Some(id) => id as isize,
                None => ERR_GENERIC,
            }
        }
        SYS_DEBUG_READ => match crate::sbi::console_getchar() {
            Some(b) => b as isize,
            None => ERR_AGAIN,
        },
        SYS_FS_LIST => {
            crate::fs::list();
            0
        }
        SYS_FS_CAT => {
            let ptr = a0 as usize;
            let len = a1 as usize;
            if len > 64 {
                return ERR_GENERIC;
            }
            let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
            match core::str::from_utf8(slice) {
                Ok(path) => {
                    if crate::fs::cat(path) {
                        0
                    } else {
                        ERR_GENERIC
                    }
                }
                Err(_) => ERR_GENERIC,
            }
        }
        SYS_EXEC => {
            let ptr = a0 as usize;
            let len = a1 as usize;
            if len == 0 || len > 64 {
                return ERR_GENERIC;
            }
            let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
            let path = match core::str::from_utf8(slice) {
                Ok(p) => p,
                Err(_) => return ERR_GENERIC,
            };
            let (name, bytes) = match crate::fs::lookup(path) {
                Some(v) => v,
                None => return ERR_GENERIC,
            };
            if bytes.len() < 4 || &bytes[..4] != b"\x7fELF" {
                return ERR_GENERIC;
            }
            let cap = match tasks.spawn(name) {
                Some(t) => t,
                None => return ERR_GENERIC,
            };
            let s = inner();
            let slot = match s.tasks.iter().position(|t| t.state == TaskState::Empty) {
                Some(i) => i,
                None => return ERR_GENERIC,
            };
            let stack = next_spawn_stack_base(slot);
            match spawn_elf_bytes(name, bytes, stack, cap) {
                Some(id) => id as isize,
                None => ERR_GENERIC,
            }
        }
        SYS_TIME => {
            /* QEMU virt mtime ≈ 10 MHz → ms = cycles / 10000. */
            (crate::timer::time_now() / 10_000) as isize
        }
        SYS_WAIT => {
            let child = a0 as usize;
            let s = inner();
            if child >= MAX_UTASKS {
                return ERR_GENERIC;
            }
            match s.tasks[child].state {
                TaskState::Zombie => {
                    s.tasks[child] = UserTask::empty();
                    0
                }
                TaskState::Empty => ERR_GENERIC,
                _ => ERR_AGAIN,
            }
        }
        _ => ERR_NOSYS,
    }
}
