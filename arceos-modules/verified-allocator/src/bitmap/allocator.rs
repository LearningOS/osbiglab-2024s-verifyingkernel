use super::block::BitmapBlock;
use crate::{AllocError, AllocResult};
use core::mem::{align_of, size_of};
#[cfg(verus_keep_ghost)]
use vstd::assert_by_contradiction;
use vstd::prelude::*;
use vstd::ptr::*;

verus! {

broadcast use crate::VERUS_layout_of_usize;

pub struct BitmapAllocator {
    current_block_addr: Option<usize>,
    current_pos: usize,
    total_bytes: usize,
    available_bytes: usize,
    #[cfg(verus_keep_ghost)]
    block_seq: Ghost<Seq<usize>>,
    #[cfg(verus_keep_ghost)]
    block_map: Tracked<Map<usize, BitmapBlock>>,
}

impl BitmapAllocator {
    spec fn is_adjacent(&self, a: usize, b: usize) -> bool {
        b == self.block_map@[a].spec_next()
    }

    spec fn len(&self) -> nat {
        self.block_seq@.len()
    }

    pub closed spec fn wf(&self) -> bool {
        match self.current_block_addr {
            None => {
                &&& self.len() == 0
                &&& self.block_seq@.to_set() =~= self.block_map@.dom()
                &&& self.total_bytes == 0
                &&& self.available_bytes == 0
            },
            Some(addr) => {
                &&& self.len() >= 1
                &&& addr == self.block_seq@.first()
                &&& self.current_pos <= usize::BITS * self.block_map@[addr].size()
                &&& self.available_bytes <= self.total_bytes
                &&& self.block_seq@.to_set() =~= self.block_map@.dom()
                &&& self.block_seq@.no_duplicates()
                &&& self.is_adjacent(self.block_seq@.last(), self.block_seq@.first())
                &&& forall|i|
                    0 <= i < self.len() - 1 ==> self.is_adjacent(
                        self.block_seq@[i],
                        #[trigger] self.block_seq@[i + 1],
                    )
                &&& forall|p|
                    self.block_map@.dom().contains(p) ==> #[trigger] self.block_map@[p].wf(p)
            },
        }
    }

    /// Create a new allocator with no memory region.
    pub const fn new() -> (r: Self)
        ensures
            r.wf(),
    {
        Self {
            current_block_addr: None,
            current_pos: 0,
            total_bytes: 0,
            available_bytes: 0,
            #[cfg(verus_keep_ghost)]
            block_seq: Ghost(Seq::empty()),
            #[cfg(verus_keep_ghost)]
            block_map: Tracked(Map::tracked_empty()),
        }
    }

    /// # Safety
    ///
    /// This function generates `PointsToRaw` to the given region out of void. It is called in the
    /// `unsafe_add_memory` and `unsafe_dealloc` functions. The callers of those functions are
    /// responsible to ensure the memory regions are safe to use.
    #[verifier(external_body)]
    proof fn unsafe_obtain_pt_for_region(start: usize, size: usize) -> (tracked pt: PointsToRaw)
        requires
            start > 0,
            start + size <= usize::MAX,
        ensures
            pt.is_range(start as int, size as int),
    {
        unimplemented!()
    }

    pub closed spec fn is_add_memory(self, old_self: Self, start: usize, size: usize) -> bool
    {
        &&& self.block_map@.dom() =~= old_self.block_map@.dom().insert(start)
        &&& self.block_map@[start].size() == (size / size_of::<usize>() - 2) / 9
    }

    /// Add a memory region [start, start + size) to the allocator.
    ///
    /// This is the safe version that requires a *pt* argument corresponding to the memory region.
    /// The *pt* token ensures that the caller owns this memory region and transfers the ownership
    /// to the allocator.
    pub fn add_memory(&mut self, start: usize, size: usize, pt: Tracked<PointsToRaw>) -> (r:
        AllocResult)
        requires
            old(self).wf(),
            start >= 1,
            start + size <= usize::MAX,
            start % align_of::<usize>() == 0,
            size >= 11 * size_of::<usize>(),
            pt@.is_range(start as int, size as int),
        ensures
            self.wf(),
            match r {
                Ok(()) => self.is_add_memory(*old(self), start, size),
                Err(AllocError::MemoryOverlap) => *self == *old(self),
                Err(AllocError::InvalidParam | AllocError::NoMemory) => false,
            },
    {
        if usize::MAX - self.total_bytes < size {
            return Err(AllocError::MemoryOverlap);
        }
        if let Some(addr) = self.current_block_addr {
            let mut p = addr;
            let ghost i = 0;
            loop
                invariant
                    0 <= i < self.len(),
                    p == self.block_seq@[i],
                    forall|j| 0 <= j < i ==> start != self.block_seq@[j],
                    *self == *old(self),
                    self.wf(),
                    size >= 1,
                    start + size <= usize::MAX,
                    addr == self.block_seq@.first(),
                ensures
                    forall|j| 0 <= j < self.len() ==> start != self.block_seq@[j],
                    p == self.block_seq@.last(),
            {
                let tracked block = self.block_map.borrow().tracked_borrow(p);
                assert(block.open_wf(p)) by { block.lemma_open_wf(p) };
                if start < p + 2 * size_of::<usize>() + 9 * BitmapBlock::get_size(p, Tracked(block))
                    * size_of::<usize>() && start + size > p {
                    return Err(AllocError::MemoryOverlap);
                }
                let next = BitmapBlock::next(p, Tracked(block));
                if next == addr {
                    proof {
                        assert_by_contradiction!(i == self.len() - 1, {
                            assert(self.block_seq@[i + 1] == self.block_seq@.first());
                        });
                    }
                    break ;
                }
                proof {
                    i = i + 1;
                }
                p = next;
            }
            let tracked block = self.block_map.borrow_mut().tracked_remove(p);
            BitmapBlock::set_next(p, Tracked(&mut block), start);
            proof {
                self.block_map.borrow_mut().tracked_insert(p, block);
            }
        }
        #[allow(unused_variables)]  // `block` is ghost code
        let block = BitmapBlock::new(start, size, self.current_block_addr, pt);
        self.current_block_addr = Some(start);
        self.current_pos = 0;
        proof {
            self.block_seq@ = self.block_seq@.insert(0, start);
            self.block_map.borrow_mut().tracked_insert(start, block.get());
        }
        #[verusfmt::skip]
        assert(forall|i| 0 <= i < self.block_seq@.len() - 1 ==>
            old(self).block_seq@[i] == self.block_seq@[i + 1]);
        assert(self.block_seq@.first() == start);
        self.total_bytes = self.total_bytes + size;
        self.available_bytes = self.available_bytes + size;
        Ok(())
    }

    /// Add a memory region [start, start + size) to the allocator.
    ///
    /// # Safety
    ///
    /// This is the unsafe version that can be used outside of Verus.
    /// The caller is responsible to ensure the memory region is safe to use.
    pub fn unsafe_add_memory(&mut self, start: usize, size: usize) -> (r: AllocResult)
        requires
            old(self).wf(),
        ensures
            self.wf(),
            match r {
                Ok(()) => self.is_add_memory(*old(self), start, size),
                Err(AllocError::InvalidParam | AllocError::MemoryOverlap) => *self == *old(self),
                Err(AllocError::NoMemory) => false,
            },
    {
        if start == 0 || usize::MAX - start < size || start % align_of::<usize>() != 0 || size < 11
            * size_of::<usize>() {
            return Err(AllocError::InvalidParam);
        }
        let pt = Tracked(Self::unsafe_obtain_pt_for_region(start, size));
        self.add_memory(start, size, pt)
    }

    /// Allocate a memory region with the given size and alignment.
    ///
    /// `Tracked<PointsToRaw>` in the return value can be used in Verus to prove the ownership of
    /// the allocated memory region, and should be used later in `dealloc`. It can be ignored if
    /// not using Verus.
    pub fn alloc(&mut self, size: usize, align: usize) -> (r: AllocResult<
        (usize, Tracked<PointsToRaw>),
    >)
        requires
            old(self).wf(),
            align >= 1,
            size >= 1,
            size % align == 0,
        ensures
            ({
                match r {
                    Err(AllocError::NoMemory) => *self =~= *old(self),
                    Err(AllocError::InvalidParam | AllocError::MemoryOverlap) => false,
                    Ok((alloc_addr, alloc_pt)) => {
                        &&& self.wf()
                        &&& alloc_addr % align == 0
                        &&& alloc_pt@.is_range(alloc_addr as int, size as int)
                    },
                }
            }),
    {
        if size > self.available_bytes {
            return Err(AllocError::NoMemory);
        }
        let current_block_addr = self.current_block_addr.unwrap();
        let mut p = current_block_addr;
        let tracked block = self.block_map.borrow_mut().tracked_remove(p);
        let result = BitmapBlock::alloc(p, Tracked(&mut block), size, align, self.current_pos);
        proof {
            self.block_map.borrow_mut().tracked_insert(p, block);
        }
        if let Some(result) = result {
            assert(block.open_wf(p)) by { block.lemma_open_wf(p) };
            self.current_pos = result.0 + size - p - 2 * size_of::<usize>();
            if size >= self.available_bytes {
                self.available_bytes = 0;
            } else {
                self.available_bytes = self.available_bytes - size;
            }
            return Ok(result);
        }
        assert(self.block_map@ =~= old(self).block_map@);
        let ghost i = 0;
        loop
            invariant
                0 <= i < self.len(),
                p == self.block_seq@[i],
                *self =~= *old(self),
                self.wf(),
                size >= 1,
                align >= 1,
                size % align == 0,
                current_block_addr == self.block_seq@.first(),
        {
            p = BitmapBlock::next(p, Tracked(self.block_map.borrow_mut().tracked_borrow(p)));
            assert(p == self.block_seq@[0] || p == self.block_seq@[i + 1]);
            let tracked block = self.block_map.borrow_mut().tracked_remove(p);
            let result = BitmapBlock::alloc(p, Tracked(&mut block), size, align, 0);
            proof {
                self.block_map.borrow_mut().tracked_insert(p, block);
                i = i + 1;
                if i == self.len() {
                    i = 0;
                }
            }
            if let Some(result) = result {
                self.current_block_addr = Some(p);
                assert(block.open_wf(p)) by { block.lemma_open_wf(p) };
                self.current_pos = result.0 + size - p - 2 * size_of::<usize>();
                proof {
                    #[verusfmt::skip]
                    if i != 0 {
                        self.block_seq@ = self.block_seq@.skip(i).add(self.block_seq@.take(i));

                        assert(forall|j| 0 <= j < i ==>
                            self.block_seq@[self.len() - i + j] ==
                            #[trigger] old(self).block_seq@[j]);

                        assert(forall|j: int| i <= j < self.len() ==>
                            self.block_seq@[j - i] ==
                            #[trigger] old(self).block_seq@[j]);

                        assert(forall|j| 0 <= j < self.len() - i - 1 ==>
                            self.is_adjacent(self.block_seq@[j], #[trigger] self.block_seq@[j + 1])
                        ) by {
                            assert(forall|j| 0 <= j < self.len() - i - 1 ==>
                                #[trigger] self.block_seq@[j] == old(self).block_seq@[i + j]);
                            assert(forall|j: int| 0 <= j < self.len() - i - 1 ==>
                                #[trigger] self.block_seq@[j + 1] == old(self).block_seq@[i + j + 1]);
                        }

                        assert(forall|j| self.len() - i <= j < self.len() - 1 ==>
                            self.is_adjacent(self.block_seq@[j], #[trigger] self.block_seq@[j + 1])
                        ) by {
                            assert(forall|j| self.len() - i <= j < self.len() - 1 ==>
                                #[trigger] self.block_seq@[j] ==
                                old(self).block_seq@[j - self.len() + i]);
                            assert(forall|j: int| self.len() - i <= j < self.len() - 1 ==>
                                #[trigger] self.block_seq@[j + 1] ==
                                old(self).block_seq@[j - self.len() + i + 1]);
                        }
                    }
                }
                if size >= self.available_bytes {
                    self.available_bytes = 0;
                } else {
                    self.available_bytes = self.available_bytes - size;
                }
                return Ok(result);
            }
            assert(self.block_map@ == old(self).block_map@);
            if p == current_block_addr {
                break ;
            }
        }
        Err(AllocError::NoMemory)
    }

    /// Deallocate the memory region with the given address and size.
    ///
    /// This is the safe version that requires a *pt* argument corresponding to the memory region.
    /// The *pt* token ensures that the caller owns this memory region and transfers the ownership
    /// back to the allocator.
    pub fn dealloc(&mut self, addr: usize, size: usize, Tracked(pt): Tracked<PointsToRaw>)
        requires
            old(self).wf(),
            addr >= 1,
            addr + size <= usize::MAX,
            pt.is_range(addr as int, size as int),
        ensures
            self.wf(),
    {
        let current_block_addr = match self.current_block_addr {
            None => return ,
            Some(p) => p,
        };
        let mut p = current_block_addr;
        let end = addr + size;
        let ghost i = 0;
        loop
            invariant
                0 <= i < self.len(),
                p == self.block_seq@[i],
                *self =~= *old(self),
                self.wf(),
                self.current_block_addr == Some(current_block_addr),
                pt.is_range(addr as int, size as int),
                end == addr + size,
        {
            let tracked block = self.block_map.borrow().tracked_borrow(p);
            assert(block.open_wf(p)) by { block.lemma_open_wf(p) };
            if addr >= p + 2 * size_of::<usize>() && end <= p + 2 * size_of::<usize>()
                + usize::BITS as usize * BitmapBlock::get_size(p, Tracked(&block)) {
                let tracked block = self.block_map.borrow_mut().tracked_remove(p);
                BitmapBlock::dealloc(p, Tracked(&mut block), addr, size, Tracked(pt));
                proof {
                    self.block_map.borrow_mut().tracked_insert(p, block);
                }
                if self.total_bytes - self.available_bytes < size {
                    self.available_bytes = self.total_bytes;
                } else {
                    self.available_bytes = self.available_bytes + size;
                }
                return ;
            }
            p = BitmapBlock::next(p, Tracked(self.block_map.borrow_mut().tracked_borrow(p)));
            if p == current_block_addr {
                return ;
            }
            proof {
                i = i + 1;
            }
        }
    }

    /// Deallocate the memory region with the given address and size.
    ///
    /// # Safety
    ///
    /// This is the unsafe version that can be used outside of Verus.
    /// The caller is responsible to ensure the memory region is safe to use.
    pub fn unsafe_dealloc(&mut self, addr: usize, size: usize)
        requires
            old(self).wf(),
        ensures
            self.wf(),
    {
        if addr == 0 || usize::MAX - addr < size {
            return ;
        }
        let pt = Tracked(Self::unsafe_obtain_pt_for_region(addr, size));
        self.dealloc(addr, size, pt)
    }

    /// Total number of bytes managed by the allocator
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Number of available bytes for allocation
    pub fn available_bytes(&self) -> usize {
        self.available_bytes
    }

    /// Number of used bytes
    pub fn used_bytes(&self) -> usize
        requires
            self.wf(),
    {
        self.total_bytes - self.available_bytes
    }
}

} // verus!
