//! DeepRoot kernel binary — RISC-V microkernel.

#![no_std]
#![no_main]

extern crate alloc;

mod boot;
mod block;
mod cap;
mod console;
mod elf;
mod fdt;
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
mod virtio_blk;

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

    /* Probe FDT before mm so memory_reg() feeds the frame allocator. */
    fdt::probe(dtb_pa);
    mm::init(hartid, dtb_pa);

    /* Map discovered MMIO windows (UART / remaining virtio) for later use. */
    if let Some(p) = fdt::get() {
        if let Some(u) = p.uart {
            mm::sv39::map_mmio_range(u.reg.base, u.reg.size.max(0x100));
        }
        for i in 0..p.virtio_count {
            let v = p.virtio[i];
            mm::sv39::map_mmio_range(v.reg.base, v.reg.size.max(mm::layout::PAGE_SIZE));
        }
        if let Some(fb) = p.framebuffer {
            mm::sv39::map_mmio_range(fb.reg.base, fb.reg.size.max(mm::layout::PAGE_SIZE));
        }
    }

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
