//! Shared-memory grants (1.14) — Frame caps + map / unmap / revoke.
//!
//! Teaching model: allocate physical page(s), mint `CapType::Frame` (badge =
//! base PA), map into task ASes, then unmap or revoke. 1.14.3 adds contiguous
//! spans for DMA (virtqueue) and Irq mint for device handoff.

use core::cell::UnsafeCell;

use crate::cap::{TaskId, TaskTable};
use crate::ledger::LEDGER;
use crate::mm::frame;
use crate::mm::layout::{PhysAddr, PAGE_SIZE};
use crate::mm::sv39;
use crate::println;
use crate::sched;
use crate::sync::SpinLock;
use deeproot_abi::{rights, CapReason, CapType, LedgerKind};

/// Shared teaching VA window (per-AS; same number in producer and peer).
pub const SHARE_VA: usize = 0x1A00_0000;
pub const MAGIC: &[u8] = b"DeepRoot 1.14 grant\n";

const MAX_MAPS: usize = 64;
const MAX_SPANS: usize = 32;
pub const MAX_SPAN_PAGES: usize = 8;

#[derive(Clone, Copy)]
struct MapEnt {
    used: bool,
    sched_id: usize,
    va: usize,
    pa: usize,
}

impl MapEnt {
    const fn empty() -> Self {
        Self {
            used: false,
            sched_id: 0,
            va: 0,
            pa: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Span {
    used: bool,
    base_pa: usize,
    npages: usize,
}

impl Span {
    const fn empty() -> Self {
        Self {
            used: false,
            base_pa: 0,
            npages: 0,
        }
    }
}

struct Maps {
    ents: [MapEnt; MAX_MAPS],
    spans: [Span; MAX_SPANS],
}

struct MapsCell(UnsafeCell<Maps>);
unsafe impl Sync for MapsCell {}

static MAP_LOCK: SpinLock = SpinLock::new();
static MAPS: MapsCell = MapsCell(UnsafeCell::new(Maps {
    ents: [MapEnt::empty(); MAX_MAPS],
    spans: [Span::empty(); MAX_SPANS],
}));

fn maps() -> &'static mut Maps {
    unsafe { &mut *MAPS.0.get() }
}

fn track_span(base_pa: usize, npages: usize) {
    let _g = MAP_LOCK.lock();
    let m = maps();
    if let Some(s) = m.spans.iter_mut().find(|s| !s.used) {
        *s = Span {
            used: true,
            base_pa,
            npages,
        };
    }
}

fn take_span(pa: usize) -> Option<(usize, usize)> {
    let _g = MAP_LOCK.lock();
    let m = maps();
    for s in m.spans.iter_mut() {
        if !s.used {
            continue;
        }
        let end = s.base_pa + s.npages * PAGE_SIZE;
        if pa >= s.base_pa && pa < end && (pa - s.base_pa) % PAGE_SIZE == 0 {
            let base = s.base_pa;
            let n = s.npages;
            *s = Span::empty();
            return Some((base, n));
        }
    }
    None
}

fn span_npages(pa: usize) -> usize {
    let _g = MAP_LOCK.lock();
    let m = maps();
    for s in m.spans.iter() {
        if s.used && s.base_pa == pa {
            return s.npages;
        }
    }
    1
}

fn track_map(sched_id: usize, va: usize, pa: usize) {
    let _g = MAP_LOCK.lock();
    let m = maps();
    if let Some(e) = m.ents.iter_mut().find(|e| !e.used) {
        *e = MapEnt {
            used: true,
            sched_id,
            va,
            pa,
        };
    }
}

fn untrack(sched_id: usize, va: usize) -> Option<usize> {
    let _g = MAP_LOCK.lock();
    let m = maps();
    for e in m.ents.iter_mut() {
        if e.used && e.sched_id == sched_id && e.va == va {
            let pa = e.pa;
            *e = MapEnt::empty();
            return Some(pa);
        }
    }
    None
}

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
    alloc_n(tasks, owner, 1)
}

/*
 * alloc_n - contiguous DMA span (1..=MAX_SPAN_PAGES); badge = base PA
 */
pub fn alloc_n(tasks: &mut TaskTable, owner: TaskId, npages: usize) -> Option<usize> {
    if npages == 0 || npages > MAX_SPAN_PAGES {
        return None;
    }
    let pa = frame::alloc_contiguous(npages)?;
    let cs = tasks.cspace_mut(owner)?;
    let r = rights::READ | rights::WRITE | rights::GRANT;
    match cs.install_copy(CapType::Frame, r, pa.as_usize() as u64, CapReason::Mint) {
        Ok(slot) => {
            track_span(pa.as_usize(), npages);
            println!(
                "grant: alloc frame pa={:#x} pages={} slot={}",
                pa.as_usize(),
                npages,
                slot
            );
            LEDGER.record(
                LedgerKind::FrameMap,
                pa.as_usize() as u32,
                slot as u32,
                npages as u32,
            );
            Some(slot)
        }
        Err(_) => {
            frame::free_contiguous(pa, npages);
            None
        }
    }
}

/*
 * phys - return Frame badge (physical address)
 */
pub fn phys(tasks: &TaskTable, owner: TaskId, slot: usize) -> Option<usize> {
    let (pa, _) = frame_pa(tasks, owner, slot)?;
    Some(pa.as_usize())
}

/*
 * map - map Frame span @slot into @sched_id starting at @va (npages pages)
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
    let npages = span_npages(pa.as_usize());
    for i in 0..npages {
        let page_va = va + i * PAGE_SIZE;
        let page_pa = PhysAddr::new(pa.as_usize() + i * PAGE_SIZE);
        sv39::map_user_in(root, page_va, page_pa, false, write);
        track_map(sched_id, page_va, page_pa.as_usize());
        LEDGER.record(
            LedgerKind::FrameMap,
            sched_id as u32,
            page_va as u32,
            page_pa.as_usize() as u32,
        );
    }
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
 * unmap - clear leaf mapping at @va in @sched_id
 */
pub fn unmap(sched_id: usize, va: usize) -> bool {
    if va == 0 || va % PAGE_SIZE != 0 {
        return false;
    }
    let Some(root) = sched::root_pa_of(sched_id) else {
        return false;
    };
    if !sv39::unmap_user_in(root, va) {
        return false;
    }
    let pa = untrack(sched_id, va).unwrap_or(0);
    println!(
        "grant: unmapped sched={} va={:#x} pa={:#x}",
        sched_id, va, pa
    );
    LEDGER.record(
        LedgerKind::FrameUnmap,
        sched_id as u32,
        va as u32,
        pa as u32,
    );
    true
}

/*
 * unmap_all_pa - tear down every tracked map of pages in [base, base+n*PAGE)
 */
fn unmap_all_span(base_pa: usize, npages: usize) -> usize {
    let mut batch = [(0usize, 0usize); MAX_MAPS];
    let mut n = 0usize;
    let end = base_pa + npages * PAGE_SIZE;
    {
        let _g = MAP_LOCK.lock();
        let m = maps();
        for e in m.ents.iter_mut() {
            if e.used && e.pa >= base_pa && e.pa < end {
                batch[n] = (e.sched_id, e.va);
                n += 1;
                *e = MapEnt::empty();
            }
        }
    }
    let mut done = 0usize;
    for i in 0..n {
        let (sid, va) = batch[i];
        if let Some(root) = sched::root_pa_of(sid) {
            if sv39::unmap_user_in(root, va) {
                done += 1;
                LEDGER.record(
                    LedgerKind::FrameUnmap,
                    sid as u32,
                    va as u32,
                    base_pa as u32,
                );
            }
        }
    }
    done
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

/*
 * mint_mmio - mint a Frame cap for a device MMIO page (badge = PA)
 *
 * Not from the RAM allocator — revoke unmaps only; never frame::free.
 */
pub fn mint_mmio(tasks: &mut TaskTable, owner: TaskId, pa: usize) -> Option<usize> {
    if pa == 0 || pa % PAGE_SIZE != 0 {
        return None;
    }
    if frame::contains(PhysAddr::new(pa)) {
        /* Refuse accidental mint of RAM as "MMIO". */
        return None;
    }
    let cs = tasks.cspace_mut(owner)?;
    let r = rights::READ | rights::WRITE | rights::GRANT;
    match cs.install_copy(CapType::Frame, r, pa as u64, CapReason::Mint) {
        Ok(slot) => {
            println!("grant: mmio frame pa={:#x} slot={}", pa, slot);
            Some(slot)
        }
        Err(_) => None,
    }
}

/*
 * mint_irq - mint CapType::Irq with badge = interrupt number (1.14.3)
 */
pub fn mint_irq(tasks: &mut TaskTable, owner: TaskId, irq: u32) -> Option<usize> {
    if irq == 0 {
        return None;
    }
    let cs = tasks.cspace_mut(owner)?;
    let r = rights::READ | rights::GRANT;
    match cs.install_copy(CapType::Irq, r, irq as u64, CapReason::Mint) {
        Ok(slot) => {
            println!("grant: irq={} slot={}", irq, slot);
            Some(slot)
        }
        Err(_) => None,
    }
}

/*
 * on_revoke_frame - after CapSpace::revoke of a Frame: unmap + free span
 *
 * Device MMIO badges skip free(). RAM spans free all contiguous pages.
 */
pub fn on_revoke_frame(pa: usize) -> usize {
    if pa == 0 || pa % PAGE_SIZE != 0 {
        return 0;
    }
    let phys = PhysAddr::new(pa);
    if !frame::contains(phys) {
        let n = unmap_all_span(pa, 1);
        println!("grant: revoked mmio pa={:#x} unmaps={}", pa, n);
        return n;
    }
    let (base, npages) = take_span(pa).unwrap_or((pa, 1));
    let n = unmap_all_span(base, npages);
    frame::free_contiguous(PhysAddr::new(base), npages);
    println!(
        "grant: revoked frame pa={:#x} pages={} unmaps={}",
        base, npages, n
    );
    n
}
