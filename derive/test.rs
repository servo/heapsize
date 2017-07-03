#[macro_use] extern crate heapsize_derive;

mod heapsize {
    pub type HeapSizeOfFn = unsafe fn(ptr: *const ()) -> usize;

    pub trait HeapSizeOf {
        fn heap_size_of_children(&self, heap_size: HeapSizeOfFn) -> usize;
    }

    impl<T> HeapSizeOf for Box<T> {
        fn heap_size_of_children(&self, _heap_size: HeapSizeOfFn) -> usize {
            ::std::mem::size_of::<T>()
        }
    }
}


#[derive(HeapSizeOf)]
struct Foo([Box<u32>; 2], Box<u8>);

#[test]
fn test() {
    use heapsize::HeapSizeOf;
    unsafe fn test_heap_size(_ptr: *const ()) -> usize {
        unreachable!()
    }
    assert_eq!(Foo([Box::new(1), Box::new(2)], Box::new(3)).heap_size_of_children(test_heap_size), 9);
}
