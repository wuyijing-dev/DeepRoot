//! Cooperative userspace task switch (Server Grove preview of 0.6).

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::cap::{TaskId, TaskTable};
use crate::ipc::EndpointTable;
use crate::mm::frame;
use crate::mm::layout::PAGE_SIZE;
use crate::mm::sv39;
use crate::println;
use crate::syscall;

pub const MAX_UTASKS: usize = 4;
pub const USER_STACK_PAGES: usize = 4;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TrapFrame {
    /* x1..x31 at offsets 1..31; x0 unused */
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Empty,
    Ready,
    Running,
    Blocked,
    Zombie,
}

pub struct UserTask {
    pub state: TaskState,
    pub name: &'static str,
    pub cap_task: TaskId,
    pub tf: TrapFrame,
    pub stack_top: usize,
}

impl UserTask {
    const fn empty() -> Self {
        Self {
            state: TaskState::Empty,
            name: "",
            cap_task: TaskId(0),
            tf: TrapFrame::zero(),
            stack_top: 0,
        }
    }
}

struct SchedInner {
    tasks: [UserTask; MAX_UTASKS],
    current: usize,
}

struct Sched(UnsafeCell<SchedInner>);
unsafe impl Sync for Sched {}

static SCHED: Sched = Sched(UnsafeCell::new(SchedInner {
    tasks: [const { UserTask::empty() }; MAX_UTASKS],
    current: 0,
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

pub fn current_id() -> usize {
    inner().current
}

pub fn current_cap_task() -> TaskId {
    let s = inner();
    s.tasks[s.current].cap_task
}

/*
 * spawn - create a U-mode task around a loaded ELF entry
 * @stack_va_base: VA where we map USER_STACK_PAGES (grow up, sp at top)
 */
pub fn spawn(
    name: &'static str,
    entry: usize,
    stack_va_base: usize,
    cap_task: TaskId,
) -> Option<usize> {
    let s = inner();
    let idx = s.tasks.iter().position(|t| t.state == TaskState::Empty)?;
    for i in 0..USER_STACK_PAGES {
        let frame = frame::alloc()?;
        sv39::map_user(stack_va_base + i * PAGE_SIZE, frame, false, true);
    }
    let stack_top = stack_va_base + USER_STACK_PAGES * PAGE_SIZE;

    /* sstatus: SPP=0 (U), SPIE=1 so interrupts enable after sret later. */
    const SSTATUS_SPIE: usize = 1 << 5;
    const SSTATUS_SUM: usize = 1 << 18;

    let mut tf = TrapFrame::zero();
    tf.sepc = entry;
    tf.sstatus = SSTATUS_SPIE | SSTATUS_SUM;
    tf.x[2] = stack_top; /* sp */

    s.tasks[idx] = UserTask {
        state: TaskState::Ready,
        name,
        cap_task,
        tf,
        stack_top,
    };
    println!(
        "sched: spawn {} id={} entry={:#x} sp={:#x}",
        name, idx, entry, stack_top
    );
    Some(idx)
}

pub fn mark_zombie(code: usize) {
    let s = inner();
    let id = s.current;
    s.tasks[id].state = TaskState::Zombie;
    println!("sched: {} exited code={}", s.tasks[id].name, code);
}

/*
 * yield_now - pick next Ready task (round-robin)
 * Returns false if only zombies remain.
 */
pub fn yield_now() -> bool {
    let s = inner();
    let start = s.current;
    for off in 1..=MAX_UTASKS {
        let i = (start + off) % MAX_UTASKS;
        if s.tasks[i].state == TaskState::Ready || s.tasks[i].state == TaskState::Running {
            if s.tasks[s.current].state == TaskState::Running {
                s.tasks[s.current].state = TaskState::Ready;
            }
            s.current = i;
            s.tasks[i].state = TaskState::Running;
            set_current_tf(&mut s.tasks[i].tf as *mut TrapFrame);
            return true;
        }
    }
    false
}

pub fn enter_first(id: usize) -> ! {
    let s = inner();
    s.current = id;
    s.tasks[id].state = TaskState::Running;
    set_current_tf(&mut s.tasks[id].tf as *mut TrapFrame);
    restore_user();
}

/*
 * restore_user - sret into current task's TrapFrame (never returns)
 */
pub fn restore_user() -> ! {
    let tf = current_tf_ptr();
    assert!(tf != 0);
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

/*
 * handle_syscall - run syscall for current U-task; may yield
 */
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
    let current = current_cap_task();
    match nr {
        SYS_YIELD => {
            let _ = yield_now();
            0
        }
        SYS_EXIT => {
            mark_zombie(a0 as usize);
            if !yield_now() {
                println!("sched: all user tasks done");
                crate::sbi::hart_suspend_idle();
                loop {
                    crate::sbi::hart_suspend_idle();
                }
            }
            0
        }
        SYS_DEBUG_WRITE => {
            /* a0=ptr a1=len — user buffer, identity via mapped U pages; enable SUM */
            let ptr = a0 as usize;
            let len = a1 as usize;
            if len > 4096 {
                return -1;
            }
            let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
            if let Ok(s) = core::str::from_utf8(slice) {
                crate::console::_print(core::format_args!("{}", s));
            }
            len as isize
        }
        SYS_IPC_CALL => {
            let mut msg = deeproot_abi::IpcMessage::with_label(a1);
            msg.words[0] = a2;
            match crate::ipc::call_from_cap(tasks, eps, current, a0 as usize, msg) {
                Ok(()) => 0,
                Err(_) => -1,
            }
        }
        _ => syscall::dispatch(tasks, eps, current, nr, a0, a1, a2, a3),
    }
}
