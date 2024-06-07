#![no_std]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate alloc;

mod definitions_t;
mod definitions_u;
mod extra;
mod impl_u;
mod spec_t;

use alloc::boxed::Box;
pub use definitions_t::Flags as NrPteFlags;
use definitions_t::{MemRegionExec, PageTableEntryExec};
use impl_u::l2_impl::{PTDir, PT};
use spec_t::mem::{PageAllocator, PageTableMemory};
use vstd::prelude::*;

pub struct NrPageTable {
    mem: spec_t::mem::PageTableMemory,
}

impl NrPageTable {
    pub fn new(alloc: Box<dyn Fn() -> usize>, dealloc: Box<dyn Fn(usize)>) -> Self {
        let mem = PageTableMemory::new(PageAllocator { alloc, dealloc });
        Self { mem }
    }

    pub fn root_paddr(&self) -> usize {
        self.mem.cr3().base
    }

    pub fn map(&mut self, vaddr: usize, paddr: usize, flags: NrPteFlags) -> Result<(), ()> {
        let pte = PageTableEntryExec {
            frame: MemRegionExec {
                base: paddr,
                size: 4096,
            },
            flags,
        };
        let mut pt = Ghost::assume_new();
        PT::map_frame(&mut self.mem, &mut pt, vaddr, pte)
    }

    pub fn unmap(&mut self, vaddr: usize) -> Result<(), ()> {
        let mut pt = Ghost::assume_new();
        PT::unmap(&mut self.mem, &mut pt, vaddr)
    }
}

verus! {

global layout usize is size == 8, align == 8;

} // verus!
