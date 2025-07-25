use crate::common::{Cr3, Register, VirtAddress};

#[repr(align(4096))]
pub struct PageTable {
    entries: [usize; 512],
}

impl !Copy for PageTable {}

impl PageTable {
    pub fn zero(&mut self) {
        self.entries = [0; 512];
    }
}

pub fn remap_flags(virt: VirtAddress, set: usize, unset: usize, phys_mem_offset: usize) {
    let indices = virt.indices();

    // L4 table
    let l4_addr = VirtAddress((Cr3.read() & 0x000f_ffff_ffff_f000) + phys_mem_offset)
        .canonicalize()
        .0;
    let l4 = unsafe { &mut *(l4_addr as *mut PageTable) };

    let l3_entry = &mut l4.entries[indices[0]];
    assert!(*l3_entry & 1 != 0, "L3 table not present");
    *l3_entry |= set;
    *l3_entry &= !unset;
    let l3 = unsafe {
        &mut *(VirtAddress((*l3_entry & 0x000f_ffff_ffff_f000) + phys_mem_offset)
            .canonicalize()
            .0 as *mut PageTable)
    };

    let l2_entry = &mut l3.entries[indices[1]];
    assert!(*l2_entry & 1 != 0, "L2 table not present");
    *l2_entry |= set;
    *l2_entry &= !unset;
    let l2 = unsafe {
        &mut *(VirtAddress((*l2_entry & 0x000f_ffff_ffff_f000) + phys_mem_offset)
            .canonicalize()
            .0 as *mut PageTable)
    };

    let l1_entry = &mut l2.entries[indices[2]];
    assert!(*l1_entry & 1 != 0, "L1 table not present");
    *l1_entry |= set;
    *l1_entry &= !unset;
    let l1 = unsafe {
        &mut *(VirtAddress((*l1_entry & 0x000f_ffff_ffff_f000) + phys_mem_offset)
            .canonicalize()
            .0 as *mut PageTable)
    };

    let entry = &mut l1.entries[indices[3]];
    *entry |= set;
    *entry &= !unset;

    // Invalider la page pour forcer le TLB à se mettre à jour
    unsafe {
        core::arch::asm!("invlpg [{}]", in(reg) virt.0, options(nostack, preserves_flags));
    }
}

pub const FORBID_EXECUTION: usize = 1 << 63;
