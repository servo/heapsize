#[macro_use]
extern crate heapsize_derive;
extern crate heapsize;

use heapsize::HeapSizeOf;

#[derive(HeapSizeOf)]
struct Tuple([Box<u32>; 2], Box<u8>);

#[test]
fn tuple_struct() {
    assert_eq!(
        Tuple([Box::new(1), Box::new(2)], Box::new(3)).heap_size_of_children(),
        9
    );
}

#[derive(HeapSizeOf)]
struct Normal {
    first: Tuple,
    second: Box<(u32, u32)>,
    #[ignore_heap_size_of = "We don't care about this"]
    ignored: Vec<Normal>,
}

#[test]
fn normal_struct() {
    let tuple = Tuple([Box::new(1), Box::new(2)], Box::new(3));
    let normal = Normal {
        first: tuple,
        second: Box::new((0, 0)),
        ignored: Vec::with_capacity(1024),
    };

    let got = normal.heap_size_of_children();
    assert_eq!(got, 9 + 2 * 4);
}
