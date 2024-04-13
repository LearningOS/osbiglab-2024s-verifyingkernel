use vstd::prelude::*;

verus! {

pub fn align_up(addr: u64, align: u64) -> (aligned: u64)
    requires
        align >= 1,
        align & sub(align, 1) == 0,
        addr + align - 1 <= u64::MAX,
    ensures
        addr <= aligned < addr + align,
        aligned & sub(align, 1) == 0,
{
    let result = (align - 1 + addr) & !(align - 1);
    assert(forall |x: u64, y: u64| #![auto] x >= y ==> x & !y >= sub(x, y)) by (bit_vector);
    assert(forall |x: u64, y: u64| #![auto] x & y <= x) by (bit_vector);
    assert(forall |x: u64, y: u64| #![auto] x & !y & y == 0) by (bit_vector);
    result
}

pub open spec fn not_intersect_with_ranges(ranges: Map<u64, u64>, start: u64, size: u64) -> bool {
    forall |p: u64| #![auto] ranges.dom().contains(p) ==> start >= p + ranges[p] || start + size <= p
}

} // verus!
