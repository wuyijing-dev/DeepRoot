//! Capability subsystem — Cap Root (0.3.x).
//!
//! Per-task CSpace, badges, revoke-by-provenance-parent, rights checks.

mod space;
mod task;

pub use space::{CapError, CapSlot, CapSpace, ProvenanceHop, CAP_SLOTS, PROVENANCE_DEPTH};
pub use task::{TaskId, TaskTable, MAX_TASKS};

use crate::println;
use crate::ledger::LEDGER;
use deeproot_abi::{rights, CapReason, CapType, LedgerKind};

/*
 * boot_demo - Cap Root worksheet exercised at boot under lesson-cap
 *
 * Builds a tiny task table, mints a root, derives a badged child, derives a
 * grandchild, then revokes the child and checks the grandchild died too.
 */
pub fn boot_demo() {
    let mut tasks = TaskTable::new();
    let init = tasks.spawn("init").expect("spawn init");
    let ping = tasks.spawn("ping").expect("spawn ping");

    let (root, ep, child, grand, child_view, grand_view, ep_badge) = {
        let init_cs = tasks.cspace_mut(init).expect("init cspace");
        let root = init_cs
            .mint_root(rights::ALL, CapType::Untyped, CapReason::BootRoot)
            .expect("mint root");
        LEDGER.record(LedgerKind::CapMint, init.0 as u32, root as u32, rights::ALL);

        /* Badged endpoint-shaped cap: badge = 0xA5A5, rights READ|IPC|GRANT. */
        let ep = init_cs
            .mint_badged(
                root,
                rights::READ | rights::IPC | rights::GRANT,
                CapType::Endpoint,
                0xA5A5,
                CapReason::Badge,
            )
            .expect("mint badged ep");
        LEDGER.record(LedgerKind::CapMint, root as u32, ep as u32, 0xA5A5);

        let child = init_cs
            .derive(
                ep,
                rights::READ | rights::IPC,
                0x00FF, /* badge &= mask → 0x00A5 */
                CapReason::Derive,
            )
            .expect("derive child");
        LEDGER.record(LedgerKind::CapDerive, ep as u32, child as u32, 0);

        let grand = init_cs
            .derive(child, rights::READ, 0xFFFF, CapReason::Derive)
            .expect("derive grandchild");
        LEDGER.record(LedgerKind::CapDerive, child as u32, grand as u32, 0);

        let child_view = init_cs.view(child).expect("child view");
        let grand_view = init_cs.view(grand).expect("grand view");
        let ep_badge = init_cs.get(ep).map(|s| s.badge).unwrap_or(0);
        (root, ep, child, grand, child_view, grand_view, ep_badge)
    };

    println!(
        "cap: task={} root={} ep={} badge={:#x} child={} badge={:#x} grand={}",
        tasks.name(init),
        root,
        ep,
        ep_badge,
        child,
        child_view.badge,
        grand
    );
    println!(
        "cap: child_rights={} grand_rights={} parent_of_grand={}",
        deeproot_abi::rights_name(child_view.rights),
        deeproot_abi::rights_name(grand_view.rights),
        grand_view.parent
    );

    /* Rights enforcement demo: WRITE not in child → BadRights. */
    match tasks
        .cspace_mut(init)
        .expect("init cspace")
        .derive(child, rights::WRITE, 0xFFFF, CapReason::Derive)
    {
        Err(CapError::BadRights) => println!("cap: enforce OK (WRITE ⊄ child rights)"),
        other => println!("cap: enforce unexpected {:?}", other),
    }

    /* Move a weak cap into ping's CSpace (copy, teaching stand-in for grant). */
    let ping_slot = {
        let weak_rights = rights::READ;
        let weak_badge = child_view.badge;
        let ping_cs = tasks.cspace_mut(ping).expect("ping cspace");
        ping_cs
            .install_copy(CapType::Endpoint, weak_rights, weak_badge, CapReason::Mint)
            .expect("install into ping")
    };
    println!(
        "cap: copied read-cap into task={} slot={}",
        tasks.name(ping),
        ping_slot
    );

    let revoked = tasks
        .cspace_mut(init)
        .expect("init cspace")
        .revoke(child)
        .expect("revoke child subtree");
    LEDGER.record(LedgerKind::CapRevoke, child as u32, revoked as u32, 0);
    println!("cap: revoke(child) removed {} slots (child+descendants)", revoked);

    let init_cs = tasks.cspace(init).expect("init cspace");
    println!(
        "cap: after revoke live(child)={} live(grand)={} live(ep)={} live(root)={}",
        init_cs.get(child).is_some(),
        init_cs.get(grand).is_some(),
        init_cs.get(ep).is_some(),
        init_cs.get(root).is_some()
    );
    println!(
        "cap: tasks alive={} (init + ping CSpaces)",
        tasks.alive_count()
    );
}
