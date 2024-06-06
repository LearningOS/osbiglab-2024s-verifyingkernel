#![no_std]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

extern crate alloc;

pub mod impl_u;
pub mod definitions_t;
pub mod definitions_u;
pub mod spec_t;
pub mod extra;

vstd::prelude::verus! {

global layout usize is size == 8, align == 8;

} // verus!
