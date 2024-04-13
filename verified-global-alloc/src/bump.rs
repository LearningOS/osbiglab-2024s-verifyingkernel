use crate::utils::*;
use builtin::*;
use builtin_macros::*;
use vstd::prelude::*;
use vstd::*;

verus! {

struct BumpAllocator {
    heap_start: u64,
    heap_end: u64,
    top: u64,
    allocation_count: u64,
    used: Tracked<Map<u64, u64>>,
}

impl BumpAllocator {
    spec fn wf(&self) -> bool {
        &&& self.heap_start <= self.top <= self.heap_end
        &&& self.used@.dom().finite()
        &&& self.used@.len() == self.allocation_count <= self.top - self.heap_start
        &&& forall|p: u64|
            #![auto]
            self.used@.dom().contains(p) ==> self.used@[p] >= 1 && p + self.used@[p] <= self.top
    }

    fn new(heap_start: u64, heap_end: u64) -> (result: Self)
        requires
            heap_start <= heap_end,
        ensures
            result.wf(),
    {
        Self {
            heap_start,
            heap_end,
            top: heap_start,
            allocation_count: 0,
            used: Tracked(Map::tracked_empty()),
        }
    }

    fn alloc(&mut self, size: u64, align: u64) -> (result: u64)
        requires
            old(self).wf(),
        ensures
            self.wf(),
            result != 0 ==> not_intersect_with_ranges(old(self).used@, result, size) && result
                & sub(align, 1) == 0 && self.used@[result] == size,
    {
        if size == 0 || align == 0 || align & (align - 1) != 0 || u64::MAX - self.top < align - 1 {
            return 0;
        }
        let alloc_start = align_up(self.top, align);
        if alloc_start >= self.heap_end || self.heap_end - alloc_start < size {
            return 0;
        }
        self.top = alloc_start + size;
        self.allocation_count = self.allocation_count + 1;
        proof {
            assert_by_contradiction!(
                !self.used@.dom().contains(alloc_start),
                { assert(alloc_start + self.used@[alloc_start] > old(self).top); }
            );
            self.used.borrow_mut().tracked_insert(alloc_start, size);
        }
        alloc_start
    }

    fn dealloc(&mut self, ptr: u64)
        requires
            old(self).wf(),
            old(self).used@.dom().contains(ptr),
        ensures
            self.wf(),
    {
        proof {
            self.used.borrow_mut().tracked_remove(ptr);
        }
        self.allocation_count = self.allocation_count - 1;
        if self.allocation_count == 0 {
            self.top = self.heap_start;
        }
    }
}

} // verus!
use alloc::alloc::{GlobalAlloc, Layout};
use spin::Mutex;

pub struct LockedBumpAllocator(Mutex<Option<BumpAllocator>>);

impl LockedBumpAllocator {
    pub const fn empty() -> Self {
        Self(Mutex::new(None))
    }

    pub unsafe fn init(&self, heap_start: u64, heap_size: u64) {
        *self.0.lock() = Some(BumpAllocator::new(
            heap_start,
            heap_start.saturating_add(heap_size),
        ));
    }
}

unsafe impl GlobalAlloc for LockedBumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(allocator) = self.0.lock().as_mut() {
            allocator.alloc(layout.size() as _, layout.align() as _) as _
        } else {
            core::ptr::null_mut()
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if let Some(allocator) = self.0.lock().as_mut() {
            allocator.dealloc(ptr as _);
        }
    }
}
