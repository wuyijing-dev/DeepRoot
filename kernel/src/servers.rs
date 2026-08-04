//! Server bring-up — per-task AS, shell, embedded spawn blobs (1.1–1.6).

use crate::cap::{CapSpace, TaskId, TaskTable};
use crate::elf;
use crate::ipc::EndpointTable;
use crate::mm::aspace::AddrSpace;
use crate::println;
use crate::sched;
use crate::trap;
use deeproot_abi::{rights, CapReason, CapType};

const PING_BADGE: u64 = 0xE001;
const CONSOLE_BADGE: u64 = 0xC001;

const STACK_INIT: usize = 0x1010_0000;
const STACK_CONSOLE: usize = 0x1110_0000;
const STACK_PING: usize = 0x1210_0000;
const STACK_SHELL: usize = 0x1610_0000;

pub static HELLO_ELF: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-hello"));

fn load_spawn(
    name: &'static str,
    bytes: &'static [u8],
    stack: usize,
    cap: TaskId,
) -> Option<usize> {
    let aspace = AddrSpace::create()?;
    let loaded = elf::load_into(&aspace, name, bytes)?;
    sched::spawn_as(name, loaded.entry, stack, cap, aspace)
}

pub fn bring_up() -> ! {
    let mut tasks = TaskTable::new();
    let mut eps = EndpointTable::new();

    let t_init = tasks.spawn("init").expect("init task");
    let t_console = tasks.spawn("console").expect("console task");
    let t_ping = tasks.spawn("ping").expect("ping task");
    let t_shell = tasks.spawn("shell").expect("shell task");
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

    let id_ping = load_spawn(
        "ping",
        include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-ping")),
        STACK_PING,
        t_ping,
    )
    .expect("spawn ping");
    let id_console = load_spawn(
        "console",
        include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-console")),
        STACK_CONSOLE,
        t_console,
    )
    .expect("spawn console");
    let id_init = load_spawn(
        "init",
        include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-init")),
        STACK_INIT,
        t_init,
    )
    .expect("spawn init");
    let id_shell = load_spawn(
        "shell",
        include_bytes!(concat!(env!("OUT_DIR"), "/deeproot-shell")),
        STACK_SHELL,
        t_shell,
    )
    .expect("spawn shell");
    let id_idle = sched::spawn_idle(t_idle).expect("spawn idle");

    println!(
        "servers: canopy ready (ping={} console={} init={} shell={} idle={})",
        id_ping, id_console, id_init, id_shell, id_idle
    );
    println!("servers: teaching path 1.1–1.6 (AS/spawn/shell/ramfs/FDT/virtio-blk)");

    trap::install_ctx(tasks, eps);
    trap::enable_user();
    sched::enter_first(id_ping);
}
