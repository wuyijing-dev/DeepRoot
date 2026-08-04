//! Trap vectors: early kernel path + U-mode TrapFrame path (0.5.x).

use crate::cap::TaskTable;
use crate::ipc::EndpointTable;
use crate::ledger::LEDGER;
use crate::println;
use crate::sched::{self, TrapFrame};
use deeproot_abi::LedgerKind;

const EXC_ECALL_U: usize = 8;
const EXC_INSTRUCTION_PAGE_FAULT: usize = 12;
const EXC_LOAD_PAGE_FAULT: usize = 13;
const EXC_STORE_PAGE_FAULT: usize = 15;

#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn early_trap_vector() {
    core::arch::naked_asm!(
        "la sp, __boot_stack_top",
        "csrr a0, scause",
        "csrr a1, sepc",
        "csrr a2, stval",
        "call early_trap",
        "1: wfi",
        "j 1b",
    );
}

#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn trap_vector() {
    core::arch::naked_asm!(
        "csrrw sp, sscratch, sp",
        "sd ra, 1*8(sp)",
        "sd gp, 3*8(sp)",
        "sd tp, 4*8(sp)",
        "sd t0, 5*8(sp)",
        "sd t1, 6*8(sp)",
        "sd t2, 7*8(sp)",
        "sd s0, 8*8(sp)",
        "sd s1, 9*8(sp)",
        "sd a0, 10*8(sp)",
        "sd a1, 11*8(sp)",
        "sd a2, 12*8(sp)",
        "sd a3, 13*8(sp)",
        "sd a4, 14*8(sp)",
        "sd a5, 15*8(sp)",
        "sd a6, 16*8(sp)",
        "sd a7, 17*8(sp)",
        "sd s2, 18*8(sp)",
        "sd s3, 19*8(sp)",
        "sd s4, 20*8(sp)",
        "sd s5, 21*8(sp)",
        "sd s6, 22*8(sp)",
        "sd s7, 23*8(sp)",
        "sd s8, 24*8(sp)",
        "sd s9, 25*8(sp)",
        "sd s10, 26*8(sp)",
        "sd s11, 27*8(sp)",
        "sd t3, 28*8(sp)",
        "sd t4, 29*8(sp)",
        "sd t5, 30*8(sp)",
        "sd t6, 31*8(sp)",
        "csrr t0, sscratch",
        "sd t0, 2*8(sp)",
        "csrr t0, sepc",
        "sd t0, 32*8(sp)",
        "csrr t0, sstatus",
        "sd t0, 33*8(sp)",
        /* Keep SIE clear while in the kernel trap path. Never set SIE in
         * S-mode: this vector assumes sscratch → U TrapFrame. */
        "li t0, 2",
        "csrc sstatus, t0",
        "la sp, __boot_stack_top",
        "call trap_handler",
        "j trap_idle",
    );
}

#[no_mangle]
extern "C" fn trap_idle() -> ! {
    loop {
        crate::sbi::hart_suspend_idle();
    }
}

#[no_mangle]
pub extern "C" fn early_trap(scause: usize, sepc: usize, stval: usize) {
    LEDGER.record(LedgerKind::Trap, scause as u32, sepc as u32, stval as u32);
    println!(
        "trap: early scause={:#x} sepc={:#x} stval={:#x}",
        scause, sepc, stval
    );
}

pub struct TrapCtx {
    pub tasks: TaskTable,
    pub eps: EndpointTable,
}

struct CtxCell(core::cell::UnsafeCell<Option<TrapCtx>>);
unsafe impl Sync for CtxCell {}
static CTX: CtxCell = CtxCell(core::cell::UnsafeCell::new(None));

pub fn install_ctx(tasks: TaskTable, eps: EndpointTable) {
    unsafe {
        *CTX.0.get() = Some(TrapCtx { tasks, eps });
    }
}

fn ctx_mut() -> &'static mut TrapCtx {
    unsafe { (*CTX.0.get()).as_mut().expect("trap ctx") }
}

#[no_mangle]
pub extern "C" fn trap_handler() {
    let tf = unsafe { &mut *(sched::current_tf_ptr() as *mut TrapFrame) };
    let mut scause: usize;
    unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) scause, options(nostack));
    }
    let code = scause & ((1usize << (usize::BITS - 1)) - 1);
    let is_interrupt = (scause as isize) < 0;

    if !is_interrupt && code == EXC_ECALL_U {
        let nr = tf.x[17];
        let a0 = tf.x[10] as u64;
        let a1 = tf.x[11] as u64;
        let a2 = tf.x[12] as u64;
        let a3 = tf.x[13] as u64;
        tf.sepc = tf.sepc.wrapping_add(4);
        let issuer = sched::current_id();
        let ctx = ctx_mut();
        let ret = sched::handle_syscall(&mut ctx.tasks, &mut ctx.eps, nr, a0, a1, a2, a3);
        /* Return value belongs to the task that issued the syscall (may have yielded). */
        sched::set_syscall_return(issuer, ret);
        sched::restore_user();
    }

    /* Supervisor timer interrupt (0.6.2) — preempt current U-task. */
    if is_interrupt && code == 5 {
        let _ = crate::timer::on_interrupt();
        sched::preempt();
        sched::restore_user();
    }

    let mut stval: usize;
    unsafe {
        core::arch::asm!("csrr {}, stval", out(reg) stval, options(nostack));
    }
    LEDGER.record(LedgerKind::Trap, scause as u32, tf.sepc as u32, stval as u32);

    if !is_interrupt
        && matches!(
            code,
            EXC_INSTRUCTION_PAGE_FAULT | EXC_LOAD_PAGE_FAULT | EXC_STORE_PAGE_FAULT
        )
    {
        let id = sched::current_id();
        println!(
            "trap: page fault scause={:#x} sepc={:#x} stval={:#x} ({}) task={} ra={:#x} sp={:#x}",
            scause,
            tf.sepc,
            stval,
            crate::mm::sv39::page_fault_hint(stval),
            id,
            tf.x[1],
            tf.x[2],
        );
    } else {
        println!(
            "trap: scause={:#x} sepc={:#x} stval={:#x} (unhandled)",
            scause, tf.sepc, stval
        );
    }
}

fn set_stvec(addr: usize) {
    unsafe {
        core::arch::asm!(
            "csrw stvec, {}",
            in(reg) addr,
            options(nomem, nostack),
        );
    }
}

pub fn init() {
    let addr = early_trap_vector as *const () as usize;
    set_stvec(addr);
    println!("trap: early stvec={:#x}", addr);
}

/*
 * enable_user - switch to TrapFrame-aware vector before first sret
 */
pub fn enable_user() {
    let addr = trap_vector as *const () as usize;
    set_stvec(addr);
    println!("trap: user stvec={:#x} (timer+ecall)", addr);
}
