use vstd::prelude::*;

verus! {

pub open spec fn spec_bext(x: usize, i: usize) -> bool
    recommends
        i < usize::BITS,
{
    (x >> i) & 1 == 1
}

#[verifier::when_used_as_spec(spec_bext)]
pub fn bext(x: usize, i: usize) -> (b: bool)
    requires
        i < usize::BITS,
    ensures
        b == spec_bext(x, i),
{
    (x >> i) & 1 == 1
}

pub broadcast proof fn lemma_bext_zero(x: usize, i: usize)
    requires
        x == 0,
        i < usize::BITS,
    ensures
        !bext(x, i),
{
    assert(forall|i: u64| #[trigger] (0 >> i) & 1 == 0) by (bit_vector);
}

pub open spec fn spec_bset(x: usize, i: usize) -> usize
    recommends
        i < usize::BITS,
{
    x | (1 << i)
}

pub broadcast proof fn lemma_bset(x: usize, i: usize)
    requires
        i < usize::BITS,
    ensures
        bext(#[trigger] bset(x, i), i),
        forall|j: usize| j < usize::BITS && j != i ==> #[trigger] bext(x, j) == bext(bset(x, i), j),
{
    assert(forall|x: u64, i: u64| i < 64 ==> #[trigger] ((x | (1 << i)) >> i) & 1 == 1)
        by (bit_vector);
    assert(forall|x: u64, i: u64, j: u64|
        i <= 64 && j <= 64 && i != j ==> #[trigger] ((x | (1 << i)) >> j) & 1 == (x >> j) & 1)
        by (bit_vector);
}

#[verifier::when_used_as_spec(spec_bset)]
pub fn bset(x: usize, i: usize) -> (y: usize)
    requires
        i < usize::BITS,
    ensures
        y == spec_bset(x, i),
{
    x | (1 << i)
}

pub open spec fn spec_bclr(x: usize, i: usize) -> usize
    recommends
        i < usize::BITS,
{
    x & !(1 << i)
}

pub broadcast proof fn lemma_bclr(x: usize, i: usize)
    requires
        i < usize::BITS,
    ensures
        !bext(#[trigger] bclr(x, i), i),
        forall|j: usize| j < usize::BITS && j != i ==> #[trigger] bext(x, j) == bext(bclr(x, i), j),
{
    assert(forall|x: u64, i: u64| i < 64 ==> #[trigger] ((x & !(1 << i)) >> i) & 1 == 0)
        by (bit_vector);
    assert(forall|x: u64, i: u64, j: u64|
        i <= 64 && j <= 64 && i != j ==> #[trigger] ((x & !(1 << i)) >> j) & 1 == (x >> j) & 1)
        by (bit_vector);
}

#[verifier::when_used_as_spec(spec_bclr)]
pub fn bclr(x: usize, i: usize) -> (y: usize)
    requires
        i < usize::BITS,
    ensures
        y == spec_bclr(x, i),
{
    x & !(1 << i)
}

#[cfg(verus_keep_ghost)]
pub broadcast group group_bitmask_lemmas {
    lemma_bext_zero,
    lemma_bset,
    lemma_bclr,
}

} // verus!
