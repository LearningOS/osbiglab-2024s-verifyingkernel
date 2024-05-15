use super::bitmask::*;
#[cfg(verus_keep_ghost)]
use core::mem::align_of;
use core::mem::size_of;
#[cfg(verus_keep_ghost)]
use vstd::arithmetic::div_mod::*;
use vstd::prelude::*;
use vstd::ptr::*;
#[cfg(verus_keep_ghost)]
use vstd::set_lib::*;

verus! {

broadcast use crate::VERUS_layout_of_usize, group_bitmask_lemmas;

pub tracked struct BitmapBlock {
    #[cfg(verus_keep_ghost)]
    size_pt: PointsTo<usize>,
    #[cfg(verus_keep_ghost)]
    next_pt: PointsTo<usize>,
    #[cfg(verus_keep_ghost)]
    user_pt: PointsToRaw,
    #[cfg(verus_keep_ghost)]
    mask_pt_map: Map<int, PointsTo<usize>>,
}

impl BitmapBlock {
    pub closed spec fn wf(&self, addr: usize) -> bool {
        &&& self.open_wf(addr)
        &&& self.pt_valid(addr)
        &&& self.masks_consistent_with_pt(addr)
    }

    pub open spec fn open_wf(&self, addr: usize) -> bool {
        addr + 2 * size_of::<usize>() + 9 * self.size() * size_of::<usize>() <= usize::MAX
    }

    pub proof fn lemma_open_wf(&self, addr: usize)
        requires
            self.wf(addr),
        ensures
            self.open_wf(addr),
    {
    }

    spec fn pt_valid(&self, addr: usize) -> bool {
        let masks_start = addr + 2 * size_of::<usize>() + self.size() * usize::BITS;
        &&& addr % align_of::<usize>() == 0
        &&& self.size_pt@.pptr == addr
        &&& self.size_pt@.value.is_some()
        &&& self.next_pt@.pptr == addr + size_of::<usize>()
        &&& self.next_pt@.value.is_some()
        &&& self.user_pt@.dom().subset_of(set_int_range(addr + 2 * size_of::<usize>(), masks_start))
        &&& self.mask_pt_map.dom() =~= set_int_range(0, self.size() as int)
        &&& forall|i: int|
            0 <= i < self.size() ==> #[trigger] self.mask_pt_map[i]@.pptr == masks_start + i
                * size_of::<usize>()
        &&& forall|i: int| 0 <= i < self.size() ==> #[trigger] self.mask_pt_map[i]@.value.is_some()
    }

    spec fn masks_consistent_with_pt(&self, addr: usize) -> bool {
        forall|i: int|
            0 <= i < usize::BITS * self.size() ==> (#[trigger] self.spec_is_free(i)
                <==> self.user_pt@.dom().contains(addr + 2 * size_of::<usize>() + i))
    }

    pub closed spec fn size(&self) -> usize {
        self.size_pt@.value.unwrap()
    }

    pub fn get_size(addr: usize, Tracked(block): Tracked<&Self>) -> (r: usize)
        requires
            block.wf(addr),
        ensures
            r == block.size(),
    {
        PPtr::from_usize(addr).read(Tracked(&block.size_pt))
    }

    pub closed spec fn spec_next(&self) -> usize {
        self.next_pt@.value.unwrap()
    }

    pub fn next(addr: usize, Tracked(block): Tracked<&Self>) -> (r: usize)
        requires
            block.wf(addr),
        ensures
            r == block.spec_next(),
    {
        PPtr::from_usize(addr + size_of::<usize>()).read(Tracked(&block.next_pt))
    }

    pub fn set_next(addr: usize, Tracked(block): Tracked<&mut Self>, next: usize)
        requires
            old(block).wf(addr),
        ensures
            block.wf(addr),
            block.spec_next() == next,
            block.size() == old(block).size(),
    {
        PPtr::from_usize(addr + size_of::<usize>()).write(Tracked(&mut block.next_pt), next);
        assert(forall|i: int|
            0 <= i < block.size() * usize::BITS ==> old(block).spec_is_free(i)
                == block.spec_is_free(i));
    }

    spec fn spec_is_free(&self, index: int) -> bool
        recommends
            0 <= index < usize::BITS * self.size(),
    {
        !bext(
            self.mask_pt_map[index / usize::BITS as int]@.value.unwrap(),
            (index % usize::BITS as int) as usize,
        )
    }

    fn is_free(masks_start: usize, Tracked(block): Tracked<&Self>, index: usize) -> (r: bool)
        requires
            masks_start - 2 * size_of::<usize>() - usize::BITS * block.size() >= 0,
            block.wf((masks_start - 2 * size_of::<usize>() - usize::BITS * block.size()) as usize),
            index < usize::BITS * block.size(),
        ensures
            r == block.spec_is_free(index as int),
    {
        let ptr = PPtr::from_usize(masks_start + index / usize::BITS as usize * size_of::<usize>());
        let pt = Tracked(block.mask_pt_map.tracked_borrow(index as int / usize::BITS as int));
        let mask = ptr.read(pt);
        !bext(mask, index % usize::BITS as usize)
    }

    fn set_allocated(masks_start: usize, Tracked(block): Tracked<&mut Self>, index: usize) -> (pt:
        Tracked<PointsToRaw>)
        requires
            ({
                let addr = masks_start - 2 * size_of::<usize>() - usize::BITS * old(block).size();
                &&& addr >= 0
                &&& old(block).wf(addr as usize)
                &&& index < usize::BITS * old(block).size()
                &&& old(block).spec_is_free(index as int)
            }),
        ensures
            ({
                let addr = (masks_start - 2 * size_of::<usize>() - usize::BITS
                    * block.size()) as usize;
                let alloc_addr = addr + 2 * size_of::<usize>() + index;
                &&& block.wf(addr)
                &&& block.size_pt =~= old(block).size_pt
                &&& block.next_pt =~= old(block).next_pt
                &&& !block.spec_is_free(index as int)
                &&& forall|i: int|
                    0 <= i < usize::BITS * block.size() && i != index ==> (
                    #[trigger] block.spec_is_free(i) <==> old(block).spec_is_free(i))
                &&& pt@.is_range(alloc_addr, 1)
            }),
    {
        let p = index / usize::BITS as usize;
        let ptr = PPtr::from_usize(masks_start + p * size_of::<usize>());
        let tracked mask_pt = block.mask_pt_map.tracked_remove(p as int);
        let mask = ptr.read(Tracked(&mask_pt));
        ptr.write(Tracked(&mut mask_pt), bset(mask, index % usize::BITS as usize));
        proof { block.mask_pt_map.tracked_insert(p as int, mask_pt) }
        let pt = Tracked(
            block.user_pt.take(set![masks_start - usize::BITS * block.size() + index]),
        );
        assert(forall|i: int|
            0 <= i < usize::BITS * block.size() && i != index ==> (#[trigger] block.spec_is_free(i)
                <==> old(block).spec_is_free(i)));
        pt
    }

    fn set_free(
        masks_start: usize,
        Tracked(block): Tracked<&mut Self>,
        index: usize,
        Tracked(pt): Tracked<PointsToRaw>,
    )
        requires
            ({
                let addr = masks_start - 2 * size_of::<usize>() - usize::BITS * old(block).size();
                let dealloc_addr = addr + 2 * size_of::<usize>() + index;
                &&& addr >= 0
                &&& old(block).wf(addr as usize)
                &&& index < usize::BITS * old(block).size()
                &&& pt.is_range(dealloc_addr, 1)
            }),
        ensures
            ({
                let addr = (masks_start - 2 * size_of::<usize>() - usize::BITS
                    * block.size()) as usize;
                &&& block.wf(addr)
                &&& block.size_pt =~= old(block).size_pt
                &&& block.next_pt =~= old(block).next_pt
                &&& block.spec_is_free(index as int)
                &&& forall|i: int|
                    0 <= i < usize::BITS * block.size() && i != index ==> (
                    #[trigger] block.spec_is_free(i) <==> old(block).spec_is_free(i))
            }),
    {
        let p = index / usize::BITS as usize;
        let ptr = PPtr::from_usize(masks_start + p * size_of::<usize>());
        let tracked mask_pt = block.mask_pt_map.tracked_remove(p as int);
        let mask = ptr.read(Tracked(&mask_pt));
        ptr.write(Tracked(&mut mask_pt), bclr(mask, index % usize::BITS as usize));
        proof {
            block.mask_pt_map.tracked_insert(p as int, mask_pt);
            block.user_pt.insert(pt);
            assert(forall|i: int|
                0 <= i < usize::BITS * block.size() && i != index ==> (
                #[trigger] block.spec_is_free(i) <==> old(block).spec_is_free(i)));
        }
    }

    pub fn new(
        addr: usize,
        total_size: usize,
        next: Option<usize>,
        Tracked(pt): Tracked<PointsToRaw>,
    ) -> (result: Tracked<Self>)
        requires
            addr % align_of::<usize>() == 0,
            total_size >= 11 * size_of::<usize>(),
            addr + total_size <= usize::MAX,
            pt.is_range(addr as int, total_size as int),
        ensures
            result@.wf(addr),
            result@.size() == (total_size / size_of::<usize>() - 2) / 9,
            result@.spec_next() == next.unwrap_or(addr),
    {
        let size = (total_size / size_of::<usize>() - 2) / 9;
        let user_start = addr + 2 * size_of::<usize>();
        let masks_start = user_start + size * usize::BITS as usize;
        let tracked (size_pt, pt) = pt.split(set_int_range(addr as int, addr + size_of::<usize>()));
        let tracked (next_pt, pt) = pt.split(
            set_int_range(addr + size_of::<usize>(), user_start as int),
        );
        let tracked (user_pt, pt) = pt.split(set_int_range(user_start as int, masks_start as int));
        let tracked (masks_pt, _) = pt.split(
            set_int_range(masks_start as int, masks_start + size * size_of::<usize>()),
        );
        let tracked size_pt = size_pt.into_typed(addr as int);
        PPtr::from_usize(addr).write(Tracked(&mut size_pt), size);
        let tracked next_pt = next_pt.into_typed(addr + size_of::<usize>());
        PPtr::from_usize(addr + size_of::<usize>()).write(
            Tracked(&mut next_pt),
            next.unwrap_or(addr),
        );
        let tracked mask_pt_map = Map::<int, PointsTo<usize>>::tracked_empty();
        let mut i = 0;
        while i < size
            invariant
                0 <= i <= size,
                masks_start % align_of::<usize>() == 0,
                masks_start + size * size_of::<usize>() <= usize::MAX,
                masks_pt.is_range(
                    masks_start + i * size_of::<usize>(),
                    (size - i) * size_of::<usize>(),
                ),
                mask_pt_map.dom() =~= set_int_range(0, i as int),
                #[verusfmt::skip]
                forall|j| 0 <= j < i ==>
                    #[trigger] mask_pt_map[j]@.pptr == masks_start + j * size_of::<usize>(),
                forall|j| 0 <= j < i ==> #[trigger] mask_pt_map[j]@.value == Some(0usize),
        {
            let p = masks_start + i * size_of::<usize>();
            let tracked (l, r) = masks_pt.split(set_int_range(p as int, p + size_of::<usize>()));
            let tracked mask_pt = l.into_typed(p as int);
            PPtr::from_usize(p).write(Tracked(&mut mask_pt), 0);
            proof {
                mask_pt_map.tracked_insert(i as int, mask_pt);
                masks_pt = r;
            }
            i += 1;
        }
        Tracked(Self { size_pt, next_pt, user_pt, mask_pt_map })
    }

    pub fn alloc(
        addr: usize,
        Tracked(block): Tracked<&mut Self>,
        size: usize,
        align: usize,
        start_pos: usize,
    ) -> (result: Option<(usize, Tracked<PointsToRaw>)>)
        requires
            old(block).wf(addr),
            size >= 1,
            align >= 1,
            size % align == 0,
            start_pos <= usize::BITS * old(block).size(),
        ensures
            ({
                match result {
                    None => *block =~= *old(block),
                    Some((alloc_addr, alloc_pt)) => {
                        &&& block.wf(addr)
                        &&& alloc_addr % align == 0
                        &&& alloc_addr >= addr + 2 * size_of::<usize>()
                        &&& alloc_addr + size <= addr + 2 * size_of::<usize>() + usize::BITS
                            * block.size()
                        &&& alloc_pt@.is_range(alloc_addr as int, size as int)
                    },
                }
            }),
            block.size() == old(block).size(),
            block.spec_next() == old(block).spec_next(),
    {
        let block_size = Self::get_size(addr, Tracked(block));
        let data_size = usize::BITS as usize * block_size;
        if size > data_size - start_pos {
            return None;
        }
        let mut combo = 0usize;
        let data_start = addr + 2 * size_of::<usize>();
        let masks_start = data_start + data_size;
        for i in start_pos..data_size
            invariant
                combo <= i - start_pos,
                forall|j| i - combo <= j < i ==> block.spec_is_free(j),
                *block =~= *old(block),
                block.wf(addr),
                data_start == addr + 2 * size_of::<usize>(),
                data_size == usize::BITS * block.size(),
                masks_start == data_start + data_size,
                align >= 1,
                size % align == 0,
        {
            if Self::is_free(masks_start, Tracked(block), i) {
                combo += 1;
                if combo >= size && (data_start + i + 1) % align == 0 {
                    let alloc_pos = i + 1 - size;
                    let alloc_addr = data_start + alloc_pos;
                    let tracked alloc_pt = PointsToRaw::empty();
                    for j in alloc_pos..i + 1
                        invariant
                            forall|k| j <= k <= i ==> block.spec_is_free(k),
                            alloc_pt.is_range(alloc_addr as int, j - alloc_pos),
                            block.wf(addr),
                            block.spec_next() == old(block).spec_next(),
                            data_start == addr + 2 * size_of::<usize>(),
                            data_size == usize::BITS * block.size(),
                            masks_start == data_start + data_size,
                            alloc_addr == data_start + alloc_pos,
                            i < data_size,
                    {
                        let Tracked(pt) = Self::set_allocated(masks_start, Tracked(block), j);
                        proof { alloc_pt = alloc_pt.join(pt) }
                    }
                    assert(alloc_addr % align == 0) by {
                        lemma_sub_mod_noop(data_start + i + 1, size as int, align as int);
                    }
                    return Some((alloc_addr, Tracked(alloc_pt)));
                }
            } else {
                combo = 0;
            }
        }
        None
    }

    pub fn dealloc(
        block_addr: usize,
        Tracked(block): Tracked<&mut Self>,
        dealloc_addr: usize,
        dealloc_size: usize,
        Tracked(dealloc_pt): Tracked<PointsToRaw>,
    )
        requires
            old(block).wf(block_addr),
            dealloc_addr >= block_addr + 2 * size_of::<usize>(),
            dealloc_addr + dealloc_size <= block_addr + 2 * size_of::<usize>() + usize::BITS * old(
                block,
            ).size(),
            dealloc_pt.is_range(dealloc_addr as int, dealloc_size as int),
        ensures
            block.wf(block_addr),
            block.size() == old(block).size(),
            block.spec_next() == old(block).spec_next(),
    {
        let user_start = block_addr + 2 * size_of::<usize>();
        let masks_start = user_start + usize::BITS as usize * Self::get_size(
            block_addr,
            Tracked(block),
        );
        let dealloc_pos = dealloc_addr - block_addr - 2 * size_of::<usize>();
        let tracked pt = dealloc_pt;
        for i in dealloc_pos..dealloc_pos + dealloc_size
            invariant
                pt.is_range(user_start + i, dealloc_pos + dealloc_size - i),
                user_start == block_addr + 2 * size_of::<usize>(),
                masks_start == user_start + usize::BITS * block.size(),
                dealloc_pos + dealloc_size <= usize::BITS * block.size(),
                block.wf(block_addr),
                block.spec_next() == old(block).spec_next(),
        {
            let byte_pt = Tracked(pt.take(set![user_start+i]));
            Self::set_free(masks_start, Tracked(block), i, byte_pt);
        }
    }
}

} // verus!
