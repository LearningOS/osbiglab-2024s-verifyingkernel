#![no_std]

mod bitmap;

use vstd::prelude::*;

verus! {

global layout usize is size == 8, align == 8;

#[derive(Debug)]
pub enum AllocError {
    InvalidParam,
    MemoryOverlap,
    NoMemory,
}

pub type AllocResult<T = ()> = Result<T, AllocError>;

pub use bitmap::BitmapAllocator;

} // verus!
