//! Server Grove + Schedule Canopy bring-up.

use crate::cap::{CapSpace, TaskTable};
use crate::elf;
use crate::ipc::EndpointTable;
use crate::println;
use crate::sched;
use crate::trap;
use deeproot_abi::{rights, CapReason, CapType};

const PING_BADGE: u64 = 0xE001;
const CONSOLE_BADGE: u64 = 0xC001;

const STACK_INIT: usize = 0x1010_0000;
const STACK_CONSOLE: usize = 0x1110_0000;
const STACK_PING: usize = 0x1210_0000;

/*
 * bring_up - load servers, spawn idle, enter U-mode under timer preemption
 */
pub fn bring_up() -> ! {
    let init_l = match elf::load("init", include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-init"))) {
        Some(e) => e,
        None => fail(),
    };
    let console_l = match elf::load(
        "console",
        include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-console")),
    ) {
        Some(e) => e,
        None => fail(),
    };
    let ping_l = match elf::load("ping", include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-ping"))) {
        Some(e) => e,
        None => fail(),
    };

    let mut tasks = TaskTable::new();
    let mut eps = EndpointTable::new();

    let t_init = tasks.spawn("init").expect("init task");
    let t_console = tasks.spawn("console").expect("console task");
    let t_ping = tasks.spawn("ping").expect("ping task");
    let t_idle = tasks.spawn("idle").expect("idle task");

    eps.create(t_ping, PING_BADGE).expect("ping ep");
    eps.create(t_console, CONSOLE_BADGE).expect("console ep");

    {
        let cs = tasks.cspace_mut(t_init).unwrap();
        *cs = CapSpace::new();
        cs.install_copy(CapType::Endpoint, rights::IPC, PING_BADGE, CapReason::Mint)
            .unwrap();
        cs.install_copy(
            CapType::Endpoint,
            rights::IPC,
            CONSOLE_BADGE,
            CapReason::Mint,
        )
        .unwrap();
    }

    let id_ping = sched::spawn("ping", ping_l.entry, STACK_PING, t_ping).expect("spawn ping");
    let id_console =
        sched::spawn("console", console_l.entry, STACK_CONSOLE, t_console).expect("spawn console");
    let id_init = sched::spawn("init", init_l.entry, STACK_INIT, t_init).expect("spawn init");
    let id_idle = sched::spawn_idle(t_idle).expect("spawn idle");

    println!(
        "servers: canopy ready (ping={} console={} init={} idle={})",
        id_ping, id_console, id_init, id_idle
    );
    println!("servers: UART via SYS_DEBUG_WRITE; timer preemption + blocking IPC on");

    trap::install_ctx(tasks, eps);
    trap::enable_user();
    sched::enter_first(id_ping);
}

fn fail() -> ! {
    println!("servers: ELF load failed — idle");
    loop {
        crate::sbi::hart_suspend_idle();
    }
}
