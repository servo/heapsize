/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data structure measurement.

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap, LinkedList, VecDeque};
#[cfg(feature = "unstable")]
use std::hash::BuildHasher;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::size_of;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::os::raw::c_void;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize};
use std::rc::Rc;

#[cfg(not(target_os = "windows"))]
extern {
    #[cfg(not(prefixed_jemalloc))]
    #[cfg_attr(any(target_os = "macos", target_os = "android"), link_name = "je_mallocx")]
    // Get the size of a heap block.
    //
    // Ideally Rust would expose a function like this in std::rt::heap, which would avoid the
    // jemalloc dependence.
    //
    // The C prototype is `je_malloc_usable_size(JEMALLOC_USABLE_SIZE_CONST void *ptr)`. On some
    // platforms `JEMALLOC_USABLE_SIZE_CONST` is `const` and on some it is empty. But in practice
    // this function doesn't modify the contents of the block that `ptr` points to, so we use
    // `*const c_void` here.
    fn malloc_usable_size(ptr: *const c_void) -> usize;

    #[cfg(prefixed_jemalloc)]
    fn je_malloc_usable_size(ptr: *const c_void) -> usize;
}

#[cfg(prefixed_jemalloc)]
use je_malloc_usable_size as malloc_usable_size;

/// A wrapper for je_malloc_usable_size that handles `EMPTY` and returns `usize`.
///
/// `unsafe` because the caller must ensure that the pointer is from jemalloc.
/// FIXME: This probably interacts badly with custom allocators:
/// https://doc.rust-lang.org/book/custom-allocators.html
#[cfg(not(target_os = "windows"))]
pub unsafe fn heap_size_of(ptr: *const c_void) -> usize {
    if ptr == 0x01 as *const c_void {
        0
    } else {
        malloc_usable_size(ptr)
    }
}

/// FIXME: Need to implement heap size support on Windows.
#[cfg(target_os = "windows")]
pub unsafe fn heap_size_of(ptr: *const c_void) -> usize {
    0
}

// The simplest trait for measuring the size of heap data structures. More complex traits that
// return multiple measurements -- e.g. measure text separately from images -- are also possible,
// and should be used when appropriate.
//
pub trait HeapSizeOf {
    /// Measure the size of any heap-allocated structures that hang off this value, but not the
    /// space taken up by the value itself (i.e. what size_of::<T> measures, more or less); that
    /// space is handled by the implementation of HeapSizeOf for Box<T> below.
    fn heap_size_of_children(&self) -> usize;
}

// There are two possible ways to measure the size of `self` when it's on the heap: compute it
// (with `::std::rt::heap::usable_size(::std::mem::size_of::<T>(), 0)`) or measure it directly
// using the heap allocator (with `heap_size_of`). We do the latter, for the following reasons.
//
// * The heap allocator is the true authority for the sizes of heap blocks; its measurement is
//   guaranteed to be correct. In comparison, size computations are error-prone. (For example, the
//   `rt::heap::usable_size` function used in some of Rust's non-default allocator implementations
//   underestimate the true usable size of heap blocks, which is safe in general but would cause
//   under-measurement here.)
//
// * If we measure something that isn't a heap block, we'll get a crash. This keeps us honest,
//   which is important because unsafe code is involved and this can be gotten wrong.
//
// However, in the best case, the two approaches should give the same results.
//
impl<T: HeapSizeOf> HeapSizeOf for Box<T> {
    fn heap_size_of_children(&self) -> usize {
        // Measure size of `self`.
        unsafe {
            heap_size_of(&**self as *const T as *const c_void) + (**self).heap_size_of_children()
        }
    }
}

impl HeapSizeOf for String {
    fn heap_size_of_children(&self) -> usize {
        unsafe {
            heap_size_of(self.as_ptr() as *const c_void)
        }
    }
}

impl<'a, T: ?Sized> HeapSizeOf for &'a T {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl<T: HeapSizeOf> HeapSizeOf for Option<T> {
    fn heap_size_of_children(&self) -> usize {
        match *self {
            None => 0,
            Some(ref x) => x.heap_size_of_children()
        }
    }
}

impl<T: HeapSizeOf, E: HeapSizeOf> HeapSizeOf for Result<T, E> {
    fn heap_size_of_children(&self) -> usize {
        match *self {
            Ok(ref x) => x.heap_size_of_children(),
            Err(ref e) => e.heap_size_of_children(),
        }
    }
}

impl<'a, B: ?Sized + ToOwned> HeapSizeOf for Cow<'a, B> where B::Owned: HeapSizeOf {
    fn heap_size_of_children(&self) -> usize {
        match *self {
            Cow::Borrowed(_) => 0,
            Cow::Owned(ref b) => b.heap_size_of_children(),
        }
    }
}

impl HeapSizeOf for () {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

impl<T: HeapSizeOf, U: HeapSizeOf> HeapSizeOf for (T, U) {
    fn heap_size_of_children(&self) -> usize {
        self.0.heap_size_of_children() + self.1.heap_size_of_children()
    }
}

impl<T: HeapSizeOf> HeapSizeOf for Arc<T> {
    fn heap_size_of_children(&self) -> usize {
        (**self).heap_size_of_children()
    }
}

impl<T: HeapSizeOf> HeapSizeOf for RefCell<T> {
    fn heap_size_of_children(&self) -> usize {
        self.borrow().heap_size_of_children()
    }
}

impl<T: HeapSizeOf + Copy> HeapSizeOf for Cell<T> {
    fn heap_size_of_children(&self) -> usize {
        self.get().heap_size_of_children()
    }
}

impl<T: HeapSizeOf> HeapSizeOf for Vec<T> {
    fn heap_size_of_children(&self) -> usize {
        self.iter().fold(
            unsafe { heap_size_of(self.as_ptr() as *const c_void) },
            |n, elem| n + elem.heap_size_of_children())
    }
}

impl<T: HeapSizeOf> HeapSizeOf for VecDeque<T> {
    fn heap_size_of_children(&self) -> usize {
        self.iter().fold(
            // FIXME: get the buffer pointer for heap_size_of(), capacity() is a lower bound:
            self.capacity() * size_of::<T>(),
            |n, elem| n + elem.heap_size_of_children())
    }
}

impl<T> HeapSizeOf for Vec<Rc<T>> {
    fn heap_size_of_children(&self) -> usize {
        // The fate of measuring Rc<T> is still undecided, but we still want to measure
        // the space used for storing them.
        unsafe {
            heap_size_of(self.as_ptr() as *const c_void)
        }
    }
}

#[cfg(feature = "unstable")]
impl<K: HeapSizeOf, V: HeapSizeOf, S> HeapSizeOf for HashMap<K, V, S>
    where K: Eq + Hash, S: BuildHasher {
    fn heap_size_of_children(&self) -> usize {
        //TODO(#6908) measure actual bucket memory usage instead of approximating
        let size = self.capacity() * (size_of::<V>() + size_of::<K>());
        self.iter().fold(size, |n, (key, value)| {
            n + key.heap_size_of_children() + value.heap_size_of_children()
        })
    }
}

#[cfg(not(feature = "unstable"))]
impl<K: HeapSizeOf, V: HeapSizeOf> HeapSizeOf for HashMap<K, V>
    where K: Eq + Hash {
    fn heap_size_of_children(&self) -> usize {
        //TODO(#6908) measure actual bucket memory usage instead of approximating
        let size = self.capacity() * (size_of::<V>() + size_of::<K>());
        self.iter().fold(size, |n, (key, value)| {
            n + key.heap_size_of_children() + value.heap_size_of_children()
        })
    }
}

// PhantomData is always 0.
impl<T> HeapSizeOf for PhantomData<T> {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

// A linked list has an overhead of two words per item.
impl<T: HeapSizeOf> HeapSizeOf for LinkedList<T> {
    fn heap_size_of_children(&self) -> usize {
        let mut size = 0;
        for item in self {
            size += 2 * size_of::<usize>() + size_of::<T>() + item.heap_size_of_children();
        }
        size
    }
}

// FIXME: Overhead for the BTreeMap nodes is not accounted for.
impl<K: HeapSizeOf, V: HeapSizeOf> HeapSizeOf for BTreeMap<K, V> {
    fn heap_size_of_children(&self) -> usize {
        let mut size = 0;
        for (key, value) in self.iter() {
            size += size_of::<(K, V)>() +
                    key.heap_size_of_children() +
                    value.heap_size_of_children();
        }
        size
    }
}

/// For use on types defined in external crates
/// with known heap sizes.
#[macro_export]
macro_rules! known_heap_size(
    ($size:expr, $($ty:ident),+) => (
        $(
            impl $crate::HeapSizeOf for $ty {
                #[inline(always)]
                fn heap_size_of_children(&self) -> usize {
                    $size
                }
            }
        )+
    );
    ($size: expr, $($ty:ident<$($gen:ident),+>),+) => (
        $(
        impl<$($gen: $crate::HeapSizeOf),+> $crate::HeapSizeOf for $ty<$($gen),+> {
            #[inline(always)]
            fn heap_size_of_children(&self) -> usize {
                $size
            }
        }
        )+
    );
);

known_heap_size!(0, char);
known_heap_size!(0, u8, u16, u32, u64, usize);
known_heap_size!(0, i8, i16, i32, i64, isize);
known_heap_size!(0, bool, f32, f64);
known_heap_size!(0, AtomicBool, AtomicIsize, AtomicUsize);
known_heap_size!(0, Ipv4Addr, Ipv6Addr);
