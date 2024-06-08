// #![verus::trusted]
// trusted:
// these are wrappers for the interface with the memory
// `check_overflow` is a proof to harden the specification, it reduces the overall
// trusted-ness of this file, but not in a quantifiable fashion; for this reason we deem
// it appropriate to exclude it from P:C accounting
use crate::definitions_t::*;
use alloc::boxed::Box;
use vstd::prelude::*;

verus! {

pub fn word_index(addr: usize) -> (res: usize)
    requires
        aligned(addr as nat, 8),
    ensures
        res as nat === word_index_spec(addr as nat),
        // Prove this equivalence to use the indexing lemmas
        res as nat === crate::definitions_t::index_from_offset(addr as nat, WORD_SIZE as nat),
        word_index_spec(addr as nat) === crate::definitions_t::index_from_offset(
            addr as nat,
            WORD_SIZE as nat,
        ),
{
    addr / WORD_SIZE
}

pub open spec fn word_index_spec(addr: nat) -> nat
    recommends
        aligned(addr, 8),
{
    addr / (WORD_SIZE as nat)
}

#[verifier(external_body)]
pub struct PageAllocator {
    pub alloc: Box<dyn Send + Sync + Fn() -> usize>,
    pub dealloc: Box<dyn Send + Sync + Fn(usize)>,
}

// FIXME: We need to allow the dirty and accessed bits to change in the memory.
// Or maybe we just specify reads to return those bits as arbitrary?
#[verifier(external_body)]
pub struct PageTableMemory {
    /// `phys_mem_ref` is the starting address of the physical memory linear mapping
    phys_virt_offset: usize,
    cr3: usize,
    page_allocator: PageAllocator,
}

impl PageTableMemory {
    #[verifier::external_body]
    pub fn new(phys_virt_offset: usize, page_allocator: PageAllocator) -> Self {
        let cr3 = (page_allocator.alloc)();
        Self { phys_virt_offset, cr3, page_allocator }
    }

    pub spec fn alloc_available_pages(self) -> nat;

    pub spec fn regions(self) -> Set<MemRegion>;

    pub spec fn region_view(self, r: MemRegion) -> Seq<u64>;

    pub open spec fn inv(self) -> bool {
        &&& forall|s1: MemRegion, s2: MemRegion|
            self.regions().contains(s1) && self.regions().contains(s2) && s1 !== s2 ==> !overlap(
                s1,
                s2,
            )
        &&& aligned(self.cr3_spec().base as nat, PAGE_SIZE as nat)
        &&& self.cr3_spec().size == PAGE_SIZE
    }

    pub open spec fn init(self) -> bool {
        &&& self.inv()
    }

    /// `cr3` returns a MemRegion whose base is the address at which the layer 0 page directory is mapped
    #[verifier(external_body)]
    pub fn cr3(&self) -> (res: MemRegionExec)
        ensures
            res === self.cr3_spec(),
    {
        MemRegionExec { base: self.cr3, size: PAGE_SIZE }
    }

    pub open spec fn cr3_spec(&self) -> MemRegionExec;

    // We assume that alloc_page never fails. In practice we can just keep a buffer of 3+ pages
    // that are allocated before we use map_frame.
    /// Allocates one page and returns its physical address
    #[verifier(external_body)]
    pub fn alloc_page(&mut self) -> (r: MemRegionExec)
        requires
            old(self).inv(),
            0 < old(self).alloc_available_pages(),
        ensures
            self.alloc_available_pages() == old(self).alloc_available_pages() - 1,
            r@.size == PAGE_SIZE,
            r@.base + PAGE_SIZE <= MAX_PHYADDR,
            aligned(r@.base, PAGE_SIZE as nat),
            !old(self).regions().contains(r@),
            self.regions() === old(self).regions().insert(r@),
            self.region_view(r@) === new_seq::<u64>(512nat, 0u64),
            forall|r2: MemRegion|
                r2 !== r@ ==> #[trigger] self.region_view(r2) === old(self).region_view(r2),
            self.cr3_spec() == old(self).cr3_spec(),
            self.inv(),
    {
        let base = (self.page_allocator.alloc)();
        unsafe { ((self.phys_virt_offset + base) as *mut u8).write_bytes(0, PAGE_SIZE) }
        MemRegionExec { base, size: PAGE_SIZE }
    }

    /// Deallocates a page
    #[verifier(external_body)]
    pub fn dealloc_page(&mut self, r: MemRegionExec)
        requires
            old(self).inv(),
            old(self).regions().contains(r@),
            aligned(r@.base, PAGE_SIZE as nat),
            r@.size == PAGE_SIZE,
        ensures
            self.regions() === old(self).regions().remove(r@),
            forall|r2: MemRegion|
                r2 !== r@ ==> #[trigger] self.region_view(r2) === old(self).region_view(r2),
            self.cr3_spec() == old(self).cr3_spec(),
            self.inv(),
    {
        (self.page_allocator.dealloc)(r.base);
    }

    #[verifier(external_body)]
    /// Write value to physical address `pbase + idx * WORD_SIZE`
    pub fn write(&mut self, pbase: usize, idx: usize, region: Ghost<MemRegion>, value: u64)
        requires
            pbase == region@.base,
            aligned(pbase as nat, WORD_SIZE as nat),
            old(self).inv(),
            old(self).regions().contains(region@),
            idx < 512,
        ensures
            self.region_view(region@) === old(self).region_view(region@).update(idx as int, value),
            forall|r: MemRegion| r !== region@ ==> self.region_view(r) === old(self).region_view(r),
            self.regions() === old(self).regions(),
            self.alloc_available_pages() == old(self).alloc_available_pages(),
            self.cr3_spec() == old(self).cr3_spec(),
    {
        let p = (self.phys_virt_offset + pbase + idx * 8) as *mut u64;
        unsafe { p.write_volatile(value) };
    }

    #[verifier(external_body)]
    /// Read value at physical address `pbase + idx * WORD_SIZE`
    pub fn read(&self, pbase: usize, idx: usize, region: Ghost<MemRegion>) -> (res: u64)
        requires
            pbase == region@.base,
            aligned(pbase as nat, WORD_SIZE as nat),
            self.regions().contains(region@),
            idx < 512,
        ensures
            res == self.spec_read(idx as nat, region@),
    {
        let p = (self.phys_virt_offset + pbase + idx * 8) as *mut u64;
        unsafe { p.read_volatile() }
    }

    pub open spec fn spec_read(self, idx: nat, region: MemRegion) -> (res: u64) {
        self.region_view(region)[idx as int]
    }
}

} // verus!
