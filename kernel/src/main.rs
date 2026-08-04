//! DeepRoot kernel binary — educational RISC-V microkernel.
//!
//! Current milestone: **0.5.x Server Grove**.

#![no_std]
#![no_main]

extern crate alloc;

mod boot;
mod cap;
mod console;
mod elf;
mod ipc;
mod ledger;
mod mm;
mod sbi;
mod sched;
mod servers;
mod syscall;
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
    println!("  RISC-V S-mode · educational capability kernel");
    println!("  remote: git@github.com:wuyijing-dev/DeepRoot.git");
    println!("");

    trap::init();
    LEDGER.record(LedgerKind::Trap, 0, 0, 1);

    #[cfg(feature = "lesson-mm")]
    {
        mm::init(hartid, dtb_pa);
    }

    #[cfg(feature = "lesson-servers")]
    {
        servers::bring_up();
    }

    #[cfg(all(feature = "lesson-cap", not(feature = "lesson-servers")))]
    {
        cap::boot_demo();
    }

    #[cfg(all(feature = "lesson-ipc", not(feature = "lesson-servers")))]
    {
        let mut tasks = cap::TaskTable::new();
        let mut eps = ipc::EndpointTable::new();
        ipc::boot_demo(&mut tasks, &mut eps);
    }

    println!("boot: idle (enable lesson-servers for Server Grove)");
    loop {
        sbi::hart_suspend_idle();
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    LEDGER.record(LedgerKind::Panic, 0, 0, 0);
    println!("KERNEL PANIC: {}", info);
    loop {
        sbi::hart_suspend_idle();
    }
}
