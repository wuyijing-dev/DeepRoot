//! CapSpace: slots, derive/mint/revoke, badges, provenance.

use deeproot_abi::{rights, CapReason, CapType, CapView};

/// Maximum slots in a task CSpace (small on purpose for worksheets).
pub const CAP_SLOTS: usize = 32;

/// How many provenance hops we retain per capability.
pub const PROVENANCE_DEPTH: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct ProvenanceHop {
    pub parent: u16,
    pub reason: CapReason,
}

#[derive(Clone, Copy, Debug)]
pub struct CapSlot {
    pub live: bool,
    pub cap_type: CapType,
    pub rights: u32,
    pub badge: u64,
    /// Direct parent slot for revoke subtree walks (`u16::MAX` = root).
    pub parent: u16,
    pub provenance: [Option<ProvenanceHop>; PROVENANCE_DEPTH],
}

impl CapSlot {
    const fn empty() -> Self {
        Self {
            live: false,
            cap_type: CapType::Null,
            rights: 0,
            badge: 0,
            parent: u16::MAX,
            provenance: [None; PROVENANCE_DEPTH],
        }
    }

    pub fn to_view(&self) -> CapView {
        let (reason, parent) = match self.provenance[0] {
            Some(h) => (h.reason as u16, h.parent),
            None => (0, self.parent),
        };
        CapView {
            live: self.live as u8,
            cap_type: self.cap_type as u8,
            _pad: 0,
            rights: self.rights,
            badge: self.badge,
            parent,
            reason,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CapError {
    NoSpace,
    BadIndex,
    BadRights,
    NotLive,
    NeedGrant,
}

pub struct CapSpace {
    slots: [CapSlot; CAP_SLOTS],
}

impl CapSpace {
    pub const fn new() -> Self {
        Self {
            slots: [CapSlot::empty(); CAP_SLOTS],
        }
    }

    pub fn get(&self, index: usize) -> Option<&CapSlot> {
        self.slots.get(index).filter(|s| s.live)
    }

    pub fn view(&self, index: usize) -> Option<CapView> {
        self.get(index).map(CapSlot::to_view)
    }

    pub fn live_count(&self) -> usize {
        self.slots.iter().filter(|s| s.live).count()
    }

    fn alloc_slot(&mut self) -> Result<usize, CapError> {
        self.slots
            .iter()
            .position(|s| !s.live)
            .ok_or(CapError::NoSpace)
    }

    /*
     * check_subset - enforce want ⊆ parent and GRANT rules (0.3.2)
     * @parent_rights: rights on the parent slot
     * @want: requested child rights
     *
     * GRANT on the child requires GRANT on the parent (cannot amplify).
     */
    fn check_subset(parent_rights: u32, want: u32) -> Result<(), CapError> {
        if want & !parent_rights != 0 {
            return Err(CapError::BadRights);
        }
        if want & rights::GRANT != 0 && parent_rights & rights::GRANT == 0 {
            return Err(CapError::BadRights);
        }
        Ok(())
    }

    fn push_provenance(
        parent_prov: [Option<ProvenanceHop>; PROVENANCE_DEPTH],
        parent_idx: usize,
        reason: CapReason,
    ) -> [Option<ProvenanceHop>; PROVENANCE_DEPTH] {
        let mut prov = parent_prov;
        for i in (1..PROVENANCE_DEPTH).rev() {
            prov[i] = prov[i - 1];
        }
        prov[0] = Some(ProvenanceHop {
            parent: parent_idx as u16,
            reason,
        });
        prov
    }

    /*
     * mint_root - create a root capability (no parent)
     */
    pub fn mint_root(
        &mut self,
        rights_mask: u32,
        cap_type: CapType,
        reason: CapReason,
    ) -> Result<usize, CapError> {
        let idx = self.alloc_slot()?;
        let mut prov = [None; PROVENANCE_DEPTH];
        prov[0] = Some(ProvenanceHop {
            parent: u16::MAX,
            reason,
        });
        self.slots[idx] = CapSlot {
            live: true,
            cap_type,
            rights: rights_mask,
            badge: 0,
            parent: u16::MAX,
            provenance: prov,
        };
        Ok(idx)
    }

    /*
     * mint_badged - create a child with an explicit badge (requires GRANT)
     * @parent: live parent index
     * @want: rights ⊆ parent
     * @cap_type: object type for the new slot
     * @badge: badge value stored on the child
     * @reason: CapReason::Badge / Mint
     *
     * seL4-like teaching rule: badge may be set freely when parent has GRANT.
     */
    pub fn mint_badged(
        &mut self,
        parent: usize,
        want: u32,
        cap_type: CapType,
        badge: u64,
        reason: CapReason,
    ) -> Result<usize, CapError> {
        let parent_slot = *self.get(parent).ok_or(CapError::NotLive)?;
        if parent_slot.rights & rights::GRANT == 0 {
            return Err(CapError::NeedGrant);
        }
        Self::check_subset(parent_slot.rights, want)?;

        let prov = Self::push_provenance(parent_slot.provenance, parent, reason);
        let idx = self.alloc_slot()?;
        self.slots[idx] = CapSlot {
            live: true,
            cap_type,
            rights: want,
            badge,
            parent: parent as u16,
            provenance: prov,
        };
        Ok(idx)
    }

    /*
     * derive - create a child; badge := parent.badge & badge_mask
     * @parent: live parent
     * @want: rights ⊆ parent
     * @badge_mask: AND mask applied to parent badge (0.3.4)
     * @reason: usually CapReason::Derive
     *
     * Derive does not require GRANT on parent (rights may only shrink).
     * Emitting GRANT on the child still needs GRANT on the parent.
     */
    pub fn derive(
        &mut self,
        parent: usize,
        want: u32,
        badge_mask: u64,
        reason: CapReason,
    ) -> Result<usize, CapError> {
        let parent_slot = *self.get(parent).ok_or(CapError::NotLive)?;
        Self::check_subset(parent_slot.rights, want)?;

        let prov = Self::push_provenance(parent_slot.provenance, parent, reason);
        let idx = self.alloc_slot()?;
        self.slots[idx] = CapSlot {
            live: true,
            cap_type: parent_slot.cap_type,
            rights: want,
            badge: parent_slot.badge & badge_mask,
            parent: parent as u16,
            provenance: prov,
        };
        Ok(idx)
    }

    /*
     * install_copy - put a detached copy into this CSpace (grant stand-in)
     *
     * Used when transferring a weak cap into another task's table before
     * real IPC grant (0.4.x) exists.
     */
    pub fn install_copy(
        &mut self,
        cap_type: CapType,
        rights_mask: u32,
        badge: u64,
        reason: CapReason,
    ) -> Result<usize, CapError> {
        let idx = self.alloc_slot()?;
        let mut prov = [None; PROVENANCE_DEPTH];
        prov[0] = Some(ProvenanceHop {
            parent: u16::MAX,
            reason,
        });
        self.slots[idx] = CapSlot {
            live: true,
            cap_type,
            rights: rights_mask,
            badge,
            parent: u16::MAX,
            provenance: prov,
        };
        Ok(idx)
    }

    /*
     * revoke - delete @index and every descendant (parent-pointer walk)
     * @index: slot to revoke
     *
     * Returns the number of slots cleared. Descendants are found by scanning
     * for `parent == index` repeatedly — O(n²) but clear for CAP_SLOTS=32.
     */
    pub fn revoke(&mut self, index: usize) -> Result<usize, CapError> {
        if self.get(index).is_none() {
            return Err(CapError::NotLive);
        }
        let mut count = 0usize;
        /* Post-order: keep deleting leaves that point into the doomed set. */
        let mut doomed = [false; CAP_SLOTS];
        doomed[index] = true;
        let mut grew = true;
        while grew {
            grew = false;
            for i in 0..CAP_SLOTS {
                if !self.slots[i].live || doomed[i] {
                    continue;
                }
                let p = self.slots[i].parent;
                if p != u16::MAX && doomed[p as usize] {
                    doomed[i] = true;
                    grew = true;
                }
            }
        }
        for i in 0..CAP_SLOTS {
            if doomed[i] && self.slots[i].live {
                self.slots[i] = CapSlot::empty();
                count += 1;
            }
        }
        Ok(count)
    }

    /*
     * has_rights - true if slot is live and contains all bits in @need
     */
    pub fn has_rights(&self, index: usize, need: u32) -> bool {
        self.get(index)
            .map(|s| s.rights & need == need)
            .unwrap_or(false)
    }
}

impl Default for CapSpace {
    fn default() -> Self {
        Self::new()
    }
}
