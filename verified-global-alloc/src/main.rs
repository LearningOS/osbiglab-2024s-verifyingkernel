#![no_std]
#![no_main]
#![feature(lang_items)]

extern crate alloc;
extern crate spin;

mod bump;
mod utils;

use bump::LockedBumpAllocator;

const HEAP_SIZE: usize = 1024 * 1024;
static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
#[global_allocator]
static ALLOCATOR: LockedBumpAllocator = LockedBumpAllocator::empty();

use alloc::vec::Vec;
use alloc::boxed::Box;

#[no_mangle]
pub extern "C" fn main() -> isize {
    unsafe {
        ALLOCATOR.init(HEAP_MEMORY.as_ptr() as _, HEAP_SIZE as _);
    }

    let x = Box::new(1);
    let px = &*x as *const _;
    drop(x);

    let mut a = Vec::new();
    for i in 1..=10 {
        a.push(i);
    }
    let sum = a.iter().sum();

    let y = Box::new(2);
    let py = &*y as *const _;
    drop(y);

    assert_ne!(px, py);
    drop(a);

    let z = Box::new(3);
    let pz = &*z as *const _;
    assert_eq!(px, pz);

    sum
}

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}
