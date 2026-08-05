//! Shared-memory grants (1.14) — Frame caps + map into task AS.
//!
//! Teaching model: allocate a physical page, mint `CapType::Frame` (badge =
//! PA), map into the caller's AS, then [`map_into`] another task so both see
//! the same bytes. Revoke/unmap is 1.14.1.

use crate::cap::{TaskId, TaskTable};
use crate::mm::frame;
use crate::mm::layout::{PhysAddr, PAGE_SIZE};
use crate::mm::sv39;
use crate::println;
use crate::sched;
use deeproot_abi::{rights, CapReason, CapType};

/// Shared teaching VA window (per-AS; same number in producer and peer).
    #[allow(dead_code)]
    pub const SHARE_VA: usize = 0x1A00_0000;
    #[allow(dead_code)]
    pub const MAGIC: &[u8] = b"DeepRoot 1.14 grant\n";

fn frame_pa(tasks: &TaskTable, owner: TaskId, slot: usize) -> Option<(PhysAddr, u32)> {
    let cs = tasks.cspace(owner)?;
    let cap = cs.get(slot)?;
    if !cap.live || cap.cap_type != CapType::Frame {
        return None;
    }
    let pa = PhysAddr::new(cap.badge as usize);
    if pa.as_usize() == 0 || pa.as_usize() % PAGE_SIZE != 0 {
        return None;
    }
    Some((pa, cap.rights))
}

/*
 * alloc - allocate one zeroed frame and mint a Frame cap into @owner
 */
pub fn alloc(tasks: &mut TaskTable, owner: TaskId) -> Option<usize> {
    let pa = frame::alloc()?;
    let cs = tasks.cspace_mut(owner)?;
    let r = rights::READ | rights::WRITE | rights::GRANT;
    match cs.install_copy(CapType::Frame, r, pa.as_usize() as u64, CapReason::Mint) {
        Ok(slot) => {
            println!(
                "grant: alloc frame pa={:#x} slot={}",
                pa.as_usize(),
                slot
            );
            Some(slot)
        }
        Err(_) => None,
    }
}

/*
 * map - map Frame @slot into @sched_id's address space at @va
 */
pub fn map(
    tasks: &TaskTable,
    owner: TaskId,
    sched_id: usize,
    slot: usize,
    va: usize,
    want_write: bool,
) -> bool {
    if va == 0 || va % PAGE_SIZE != 0 {
        return false;
    }
    let Some((pa, rights_bits)) = frame_pa(tasks, owner, slot) else {
        return false;
    };
    if rights_bits & rights::READ == 0 {
        return false;
    }
    let write = want_write && (rights_bits & rights::WRITE != 0);
    let Some(root) = sched::root_pa_of(sched_id) else {
        return false;
    };
    sv39::map_user_in(root, va, pa, false, write);
    unsafe {
        core::arch::asm!("sfence.vma", options(nostack));
    }
    true
}

/*
 * map_into - map caller's Frame into another task (cap-mediated share)
 */
pub fn map_into(
    tasks: &TaskTable,
    owner: TaskId,
    target_sched: usize,
    slot: usize,
    va: usize,
    want_write: bool,
) -> bool {
    map(tasks, owner, target_sched, slot, va, want_write)
}

/*
 * grant_cap - install a (possibly weaker) Frame copy into @target's CSpace
 */
pub fn grant_cap(
    tasks: &mut TaskTable,
    owner: TaskId,
    target: TaskId,
    slot: usize,
    want_write: bool,
) -> Option<usize> {
    let (pa, rights_bits) = frame_pa(tasks, owner, slot)?;
    if rights_bits & rights::GRANT == 0 && rights_bits & rights::READ == 0 {
        /* Prefer GRANT; allow READ-only share without GRANT for teaching. */
    }
    if rights_bits & rights::READ == 0 {
        return None;
    }
    let mut r = rights::READ;
    if want_write && rights_bits & rights::WRITE != 0 {
        r |= rights::WRITE;
    }
    let cs = tasks.cspace_mut(target)?;
    cs.install_copy(CapType::Frame, r, pa.as_usize() as u64, CapReason::Derive)
        .ok()
}
