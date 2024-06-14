#![no_std]

use core::fmt::{self, Debug, Formatter, LowerHex, UpperHex};
use core::ops::{Add, AddAssign, Sub, SubAssign};
#[cfg(verus_keep_ghost)]
use vstd::arithmetic::div_mod::*;
#[cfg(verus_keep_ghost)]
use vstd::layout::*;
use vstd::prelude::*;

verus! {

global layout usize is size == 8, align == 8;

pub const PAGE_SIZE_4K: usize = 0x1000;

proof fn lemma_4k_is_pow2()
    ensures
        is_power_2(PAGE_SIZE_4K as int),
{
    reveal_with_fuel(is_power_2, 13)
}

proof fn lemma_power_2_mod_and(x: usize, y: usize)
    requires
        is_power_2(y as int),
        x & sub(y, 1) == 0,
    ensures
        x % y == 0,
    decreases y,
{
    if y != 1 {
        assert((x / 2) % (y / 2) == 0) by {
            assert(forall|a: u64, b: u64| a & b == 0 ==> (a / 2) & (b / 2) == 0) by (bit_vector);
            assert(y / 2 - 1 == (y - 1) / 2);
            lemma_power_2_mod_and(x / 2, y / 2);
        }
        assert(x == (x / 2) * 2) by {
            assert(forall|a: u64, b: u64| b % 2 == 0 && a & sub(b, 1) == 0 ==> a % 2 == 0)
                by (bit_vector);
        };
        lemma_truncate_middle(x as int / 2, 2, y as int / 2);
    }
}

pub open spec fn spec_align_down(addr: usize, align: usize) -> usize {
    addr & !sub(align, 1)
}

proof fn lemma_align_down(addr: usize, align: usize)
    requires
        is_power_2(align as int),
    ensures
        addr - align < align_down(addr, align) <= addr,
        align_down(addr, align) % align == 0,
{
    assert(forall|a: u64, b: u64| a >= b ==> a & !b >= sub(a, b)) by (bit_vector);
    assert(forall|a: u64, b: u64| a & !b <= a) by (bit_vector);
    assert(forall|a: u64, b: u64| #[trigger] (a & !b) & b == 0) by (bit_vector);
    lemma_power_2_mod_and(align_down(addr, align), align);
}

#[verifier::when_used_as_spec(spec_align_down)]
pub const fn align_down(addr: usize, align: usize) -> (r: usize)
    requires
        is_power_2(align as int),
    ensures
        addr - align < r <= addr,
        r % align == 0,
{
    proof { lemma_align_down(addr, align) }
    addr & !(align - 1)
}

pub const fn align_up(addr: usize, align: usize) -> (r: usize)
    requires
        addr + align <= usize::MAX,
        is_power_2(align as int),
    ensures
        addr <= r < addr + align,
        r % align == 0,
{
    align_down(addr + align - 1, align)
}

pub const fn align_offset(addr: usize, align: usize) -> (r: usize)
    requires
        is_power_2(align as int),
    ensures
        r == addr - align_down(addr, align),
{
    assert(forall|a: u64, b: u64| a == add(a & b, a & !b)) by (bit_vector);
    assert(forall|a: u64, b: u64| a & b <= sub(u64::MAX, a & !b)) by (bit_vector);
    addr & (align - 1)
}

pub const fn is_aligned(addr: usize, align: usize) -> (r: bool)
    requires
        is_power_2(align as int),
    ensures
        r == (addr % align == 0),
{
    let offset = align_offset(addr, align);
    proof {
        lemma_align_down(addr, align);
        lemma_small_mod(offset as nat, align as nat);
        lemma_add_mod_noop(addr - offset, offset as int, align as int);
    }
    offset == 0
}

pub const fn align_down_4k(addr: usize) -> (r: usize)
    ensures
        addr - PAGE_SIZE_4K < r <= addr,
        r % PAGE_SIZE_4K == 0,
{
    proof { lemma_4k_is_pow2() }
    align_down(addr, PAGE_SIZE_4K)
}

pub const fn align_up_4k(addr: usize) -> (r: usize)
    requires
        addr <= usize::MAX - PAGE_SIZE_4K,
    ensures
        addr <= r < addr + PAGE_SIZE_4K,
        r % PAGE_SIZE_4K == 0,
{
    proof { lemma_4k_is_pow2() }
    align_up(addr, PAGE_SIZE_4K)
}

pub const fn align_offset_4k(addr: usize) -> (r: usize)
    ensures
        r == addr - align_down(addr, PAGE_SIZE_4K),
{
    proof { lemma_4k_is_pow2() }
    align_offset(addr, PAGE_SIZE_4K)
}

pub const fn is_aligned_4k(addr: usize) -> (r: bool)
    ensures
        r == (addr % PAGE_SIZE_4K == 0),
{
    proof { lemma_4k_is_pow2() }
    is_aligned(addr, PAGE_SIZE_4K)
}

#[repr(transparent)]
#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(usize);

#[repr(transparent)]
#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(usize);

impl PhysAddr {
    pub closed spec fn view(self) -> usize {
        self.0
    }

    pub const fn from(addr: usize) -> (r: Self)
        ensures
            r@ == addr,
    {
        Self(addr)
    }

    pub const fn as_usize(self) -> (r: usize)
        ensures
            r == self@,
    {
        self.0
    }

    pub const fn align_down(self, align: usize) -> (r: Self)
        requires
            is_power_2(align as int),
        ensures
            self@ - align < r@ <= self@,
            r@ % align == 0,
    {
        Self(align_down(self.0, align))
    }

    pub const fn align_up(self, align: usize) -> (r: Self)
        requires
            self@ + align <= usize::MAX,
            is_power_2(align as int),
        ensures
            self@ <= r@ < self@ + align,
            r@ % align == 0,
    {
        Self(align_up(self.0, align))
    }

    pub const fn align_offset(self, align: usize) -> (r: usize)
        requires
            is_power_2(align as int),
        ensures
            r == self@ - align_down(self@, align),
    {
        align_offset(self.0, align)
    }

    pub const fn is_aligned(self, align: usize) -> (r: bool)
        requires
            is_power_2(align as int),
        ensures
            r@ == (self@ % align == 0),
    {
        is_aligned(self.0, align)
    }

    pub const fn align_down_4k(self) -> (r: Self)
        ensures
            self@ - PAGE_SIZE_4K < r@ <= self@,
            r@ % PAGE_SIZE_4K == 0,
    {
        Self(align_down_4k(self.0))
    }

    pub const fn align_up_4k(self) -> (r: Self)
        requires
            self@ + PAGE_SIZE_4K <= usize::MAX,
        ensures
            self@ <= r@ < self@ + PAGE_SIZE_4K,
            r@ % PAGE_SIZE_4K == 0,
    {
        Self(align_up_4k(self.0))
    }

    pub const fn align_offset_4k(self) -> (r: usize)
        ensures
            r == self@ - align_down(self@, PAGE_SIZE_4K),
    {
        align_offset_4k(self.0)
    }

    pub const fn is_aligned_4k(self) -> (r: bool)
        ensures
            r == (self@ % PAGE_SIZE_4K == 0),
    {
        is_aligned_4k(self.0)
    }
}

impl VirtAddr {
    pub closed spec fn view(self) -> usize {
        self.0
    }

    #[verifier::external_body]
    pub const fn as_ptr(self) -> (r: *const u8)
        ensures
            r@.addr == self@,
    {
        self.0 as *const u8
    }

    #[verifier::external_body]
    pub const fn as_mut_ptr(self) -> (r: *mut u8)
        ensures
            r@.addr == self@,
    {
        self.0 as *mut u8
    }

    pub const fn from(addr: usize) -> (r: Self)
        ensures
            r@ == addr,
    {
        Self(addr)
    }

    pub const fn as_usize(self) -> (r: usize)
        ensures
            r == self@,
    {
        self.0
    }

    pub const fn align_down(self, align: usize) -> (r: Self)
        requires
            is_power_2(align as int),
        ensures
            self@ - align < r@ <= self@,
            r@ % align == 0,
    {
        Self(align_down(self.0, align))
    }

    pub const fn align_up(self, align: usize) -> (r: Self)
        requires
            self@ + align <= usize::MAX,
            is_power_2(align as int),
        ensures
            self@ <= r@ < self@ + align,
            r@ % align == 0,
    {
        Self(align_up(self.0, align))
    }

    pub const fn align_offset(self, align: usize) -> (r: usize)
        requires
            is_power_2(align as int),
        ensures
            r == self@ - align_down(self@, align),
    {
        align_offset(self.0, align)
    }

    pub const fn is_aligned(self, align: usize) -> (r: bool)
        requires
            is_power_2(align as int),
        ensures
            r@ == (self@ % align == 0),
    {
        is_aligned(self.0, align)
    }

    pub const fn align_down_4k(self) -> (r: Self)
        ensures
            self@ - PAGE_SIZE_4K < r@ <= self@,
            r@ % PAGE_SIZE_4K == 0,
    {
        Self(align_down_4k(self.0))
    }

    pub const fn align_up_4k(self) -> (r: Self)
        requires
            self@ + PAGE_SIZE_4K <= usize::MAX,
        ensures
            self@ <= r@ < self@ + PAGE_SIZE_4K,
            r@ % PAGE_SIZE_4K == 0,
    {
        Self(align_up_4k(self.0))
    }

    pub const fn align_offset_4k(self) -> (r: usize)
        ensures
            r == self@ - align_down(self@, PAGE_SIZE_4K),
    {
        align_offset_4k(self.0)
    }

    pub const fn is_aligned_4k(self) -> (r: bool)
        ensures
            r == (self@ % PAGE_SIZE_4K == 0),
    {
        is_aligned_4k(self.0)
    }
}

impl From<usize> for PhysAddr {
    fn from(addr: usize) -> (r: Self)
        ensures
            r@ == addr,
    {
        Self(addr)
    }
}

impl From<usize> for VirtAddr {
    fn from(addr: usize) -> (r: Self)
        ensures
            r@ == addr,
    {
        Self(addr)
    }
}

impl From<PhysAddr> for usize {
    fn from(addr: PhysAddr) -> (r: usize)
        ensures
            r == addr@,
    {
        addr.0
    }
}

impl From<VirtAddr> for usize {
    fn from(addr: VirtAddr) -> (r: usize)
        ensures
            r == addr@,
    {
        addr.0
    }
}

impl Add<usize> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> (r: Self)
        requires
            self@ + rhs <= usize::MAX,
        ensures
            r@ == self@ + rhs,
    {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for PhysAddr {
    fn add_assign(&mut self, rhs: usize)
        requires
            old(self)@ + rhs <= usize::MAX,
        ensures
            self@ == old(self)@ + rhs,
    {
        *self = Self(self.0 + rhs);
    }
}

impl Sub<usize> for PhysAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> (r: Self)
        requires
            self@ >= rhs,
        ensures
            r@ == self@ - rhs,
    {
        Self(self.0 - rhs)
    }
}

impl SubAssign<usize> for PhysAddr {
    fn sub_assign(&mut self, rhs: usize)
        requires
            old(self)@ >= rhs,
        ensures
            self@ == old(self)@ - rhs,
    {
        *self = Self(self.0 - rhs);
    }
}

impl Add<usize> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> (r: Self)
        requires
            self@ + rhs <= usize::MAX,
        ensures
            r@ == self@ + rhs,
    {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for VirtAddr {
    fn add_assign(&mut self, rhs: usize)
        requires
            old(self)@ + rhs <= usize::MAX,
        ensures
            self@ == old(self)@ + rhs,
    {
        *self = Self(self.0 + rhs);
    }
}

impl Sub<usize> for VirtAddr {
    type Output = Self;

    fn sub(self, rhs: usize) -> (r: Self)
        requires
            self@ >= rhs,
        ensures
            r@ == self@ - rhs,
    {
        Self(self.0 - rhs)
    }
}

impl SubAssign<usize> for VirtAddr {
    fn sub_assign(&mut self, rhs: usize)
        requires
            old(self)@ >= rhs,
        ensures
            self@ == old(self)@ - rhs,
    {
        *self = Self(self.0 - rhs);
    }
}

} // verus!
impl Debug for PhysAddr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}

impl LowerHex for PhysAddr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#x}", self.0))
    }
}

impl UpperHex for PhysAddr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("PA:{:#X}", self.0))
    }
}

impl LowerHex for VirtAddr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#x}", self.0))
    }
}

impl UpperHex for VirtAddr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("VA:{:#X}", self.0))
    }
}
