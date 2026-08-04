//! DeepRoot kernel binary — RISC-V microkernel.

#![no_std]
#![no_main]

extern crate alloc;

mod boot;
mod block;
mod cap;
mod console;
mod elf;
mod fs;
mod ipc;
mod ledger;
mod mm;
mod sbi;
mod sched;
mod servers;
mod syscall;
mod timer;
mod trap;
mod version;

use deeproot_abi::LedgerKind;
use ledger::LEDGER;

#[no_mangle]
pub extern "C" fn kernel_main(hartid: usize, dtb_pa: usize) -> ! {
    ledger::init();
    LEDGER.record(LedgerKind::Boot, 0, 0, 0);

    println!("");
    println!("  DeepRoot microkernel {}", version::version_string());
    println!("  RISC-V S-mode · capability microkernel");
    println!("  remote: git@github.com:wuyijing-dev/DeepRoot.git");
    println!("");

    trap::init();
    LEDGER.record(LedgerKind::Trap, 0, 0, 1);

    mm::init(hartid, dtb_pa);
    block::init();
    timer::init(hartid);
    servers::bring_up();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    LEDGER.record(LedgerKind::Panic, 0, 0, 0);
    println!("KERNEL PANIC: {}", info);
    loop {
        sbi::hart_suspend_idle();
    }
}
