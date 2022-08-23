//! A slab allocator implementation for objects less than a page-size (4 KiB or 2MiB).
//!
//! # Overview
//!
//! The organization is as follows:
//!
//!  * A `ZoneAllocator` manages many `SCAllocator` and can
//!    satisfy requests for different allocation sizes.
//!  * A `SCAllocator` allocates objects of exactly one size.
//!    It stores the objects and meta-data in one or multiple `AllocablePage` objects.
//!  * A trait `AllocablePage` that defines the page-type from which we allocate objects.
//!
//! Lastly, it provides two default `AllocablePage` implementations `ObjectPage` and `LargeObjectPage`:
//!  * A `ObjectPage` that is 4 KiB in size and contains allocated objects and associated meta-data.
//!  * A `LargeObjectPage` that is 2 MiB in size and contains allocated objects and associated meta-data.
//!
//!
//! # Implementing GlobalAlloc
//! See the [global alloc](https://github.com/gz/rust-slabmalloc/tree/master/examples/global_alloc.rs) example.
#![allow(unused_features)]
#![cfg_attr(feature = "unstable", feature(const_mut_refs))]
#![cfg_attr(test, feature(prelude_import, test, c_void_variant, core_intrinsics))]
#![no_std]
#![crate_name = "slabmalloc"]
#![crate_type = "lib"]

// The x86-64 platform specific code.
#[cfg(all(target_arch = "x86_64"))]
#[path = "arch/x86/mod.rs"]
pub mod arch;

// The aarch64 platform specific code.
#[cfg(all(target_arch = "aarch64"))]
#[path = "arch/aarch64/mod.rs"]
pub mod arch;

mod pages;
mod sc;
mod zone;

pub use arch::*;
pub use pages::*;
pub use sc::*;
pub use zone::*;

#[cfg(test)]
#[macro_use]
extern crate std;
#[cfg(test)]
extern crate test;

#[cfg(test)]
mod tests;

use core::alloc::Layout;
use core::fmt;
use core::mem;
use core::ptr::{self, NonNull};

use log::trace;

/// Error that can be returned for `allocation` and `deallocation` requests.
#[derive(Debug)]
pub enum AllocationError {
    /// Can't satisfy the allocation request for Layout because the allocator
    /// does not have enough memory (you may be able to `refill` it).
    OutOfMemory,
    /// Allocator can't deal with the provided size of the Layout.
    InvalidLayout,
}

pub unsafe trait Allocator<'a> {
    fn allocate(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocationError>;
    fn deallocate(&mut self, ptr: NonNull<u8>, layout: Layout) -> Result<(), AllocationError>;
    unsafe fn refill_large(
        &mut self,
        layout: Layout,
        new_page: &'a mut LargeObjectPage<'a>,
    ) -> Result<(), AllocationError>;
    unsafe fn refill(
        &mut self,
        layout: Layout,
        new_page: &'a mut ObjectPage<'a>,
    ) -> Result<(), AllocationError>;
}
