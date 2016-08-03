/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(plugin, custom_derive)]
#![plugin(heapsize_plugin)]

extern crate heapsize;

use heapsize::HeapSizeOf;

#[derive(Clone, Copy)]
struct Four;
impl HeapSizeOf for Four {
    fn heap_size_of_children(&self) -> usize {
        4
    }
}

struct Seven;
impl HeapSizeOf for Seven {
    fn heap_size_of_children(&self) -> usize {
        7
    }
}

#[derive(HeapSizeOf)]
struct Eight(Four, Four, bool, bool, bool);

#[derive(HeapSizeOf)]
enum EightOrFour {
    Eight(Eight),
    Four(Four),
    Zero(u8)
}

#[derive(HeapSizeOf)]
struct FourScoreAndSeven([Four; 20], Seven);

// iterator traits aren't directly implemented on larger arrays
#[derive(HeapSizeOf)]
struct SoMuchFour([Four; 200]);

#[test]
fn test_plugin() {
    //-----------------------------------------------------------------------
    // Test the HeapSizeOf auto-deriving.

    assert_eq!(Four.heap_size_of_children(), 4);
    let eight = Eight(Four, Four, true, true, true);
    assert_eq!(eight.heap_size_of_children(), 8);
    assert_eq!(EightOrFour::Eight(eight).heap_size_of_children(), 8);
    assert_eq!(EightOrFour::Four(Four).heap_size_of_children(), 4);
    assert_eq!(EightOrFour::Zero(1).heap_size_of_children(), 0);
    assert_eq!(FourScoreAndSeven([Four; 20], Seven).heap_size_of_children(), 87);
    assert_eq!(SoMuchFour([Four; 200]).heap_size_of_children(), 800);

}
