/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(feature= "unstable", feature(alloc, heap_api))]

extern crate heapsize;

use heapsize::{HeapSizeOf, heap_size_of};
use std::os::raw::c_void;

pub const EMPTY: *mut () = 0x1 as *mut ();

#[cfg(feature = "unstable")]
mod unstable {
    extern crate alloc;

    use heapsize::heap_size_of;
    use std::os::raw::c_void;

    #[test]
    fn check_empty() {
        assert_eq!(::EMPTY, alloc::heap::EMPTY);
    }

    #[test]
    fn test_alloc() {
        unsafe {
            // A 64 byte request is allocated exactly.
            let x = alloc::heap::allocate(64, 0);
            assert_eq!(heap_size_of(x as *const c_void), 64);
            alloc::heap::deallocate(x, 64, 0);

            // A 255 byte request is rounded up to 256 bytes.
            let x = alloc::heap::allocate(255, 0);
            assert_eq!(heap_size_of(x as *const c_void), 256);
            alloc::heap::deallocate(x, 255, 0);

            // A 1MiB request is allocated exactly.
            let x = alloc::heap::allocate(1024 * 1024, 0);
            assert_eq!(heap_size_of(x as *const c_void), 1024 * 1024);
            alloc::heap::deallocate(x, 1024 * 1024, 0);
        }
    }
}

#[test]
fn test_heap_size() {

    // Note: jemalloc often rounds up request sizes. However, it does not round up for request
    // sizes of 8 and higher that are powers of two. We take advantage of knowledge here to make
    // the sizes of various heap-allocated blocks predictable.

    //-----------------------------------------------------------------------
    // Start with basic heap block measurement.

    unsafe {
        // EMPTY is the special non-null address used to represent zero-size allocations.
        assert_eq!(heap_size_of(EMPTY as *const c_void), 0);
    }

    //-----------------------------------------------------------------------
    // Test HeapSizeOf implementations for various built-in types.

    // Not on the heap; 0 bytes.
    let x = 0i64;
    assert_eq!(x.heap_size_of_children(), 0);

    // An i64 is 8 bytes.
    let x = Box::new(0i64);
    assert_eq!(x.heap_size_of_children(), 8);

    // An ascii string with 16 chars is 16 bytes in UTF-8.
    assert_eq!(String::from("0123456789abcdef").heap_size_of_children(), 16);

    // Not on the heap.
    let x: Option<i32> = None;
    assert_eq!(x.heap_size_of_children(), 0);

    // Not on the heap.
    let x = Some(0i64);
    assert_eq!(x.heap_size_of_children(), 0);

    // The `Some` is not on the heap, but the Box is.
    let x = Some(Box::new(0i64));
    assert_eq!(x.heap_size_of_children(), 8);

    // Not on the heap.
    let x = ::std::sync::Arc::new(0i64);
    assert_eq!(x.heap_size_of_children(), 0);

    // The `Arc` is not on the heap, but the Box is.
    let x = ::std::sync::Arc::new(Box::new(0i64));
    assert_eq!(x.heap_size_of_children(), 8);

    // Zero elements, no heap storage.
    let x: Vec<i64> = vec![];
    assert_eq!(x.heap_size_of_children(), 0);

    // Four elements, 8 bytes per element.
    let x = vec![0i64, 1i64, 2i64, 3i64];
    assert_eq!(x.heap_size_of_children(), 32);
}
