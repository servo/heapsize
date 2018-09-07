#[macro_use]
extern crate synstructure;
#[macro_use]
extern crate quote;
extern crate proc_macro2;
extern crate syn;

use syn::{Field, Meta, MetaList, MetaNameValue, Path, Type};

use synstructure::Structure;

fn heapsizeof_derive(mut s: Structure) -> proc_macro2::TokenStream {
    let body =
        s.filter(|bi| !should_ignore_field(bi.ast())).each(|bi| {
            match bi.ast().ty {
                Type::Array(_) => quote!{
                    for item in #bi.iter() {
                        sum += item.heap_size_of_children();
                    }
                },
                _ => quote!{ sum += #bi.heap_size_of_children(); },
            }
        });

    s.gen_impl(quote! {
        extern crate heapsize;

        gen impl heapsize::HeapSizeOf for @Self {
            fn heap_size_of_children(&self) -> usize {
                let mut sum = 0;

                match *self { #body }

                sum
            }
        }
    })
}

decl_derive!([HeapSizeOf, attributes(ignore_heap_size_of)] => heapsizeof_derive);

const PANIC_MSG: &str = "#[ignore_heap_size_of] should have an explanation, \
                         e.g. #[ignore_heap_size_of = \"because reasons\"]";

fn should_ignore_field(ast: &Field) -> bool {
    for attr in &ast.attrs {
        if pretty_path(&attr.path) == "ignore_heap_size_of" {
            match attr.interpret_meta().unwrap() {
                Meta::Word(ref ident)
                | Meta::List(MetaList { ref ident, .. })
                    if ident == "ignore_heap_size_of" =>
                {
                    panic!("{}", PANIC_MSG);
                }
                Meta::NameValue(MetaNameValue { ref ident, .. })
                    if ident == "ignore_heap_size_of" =>
                {
                    return true
                }
                _ => {}
            }
        }
    }

    false
}

fn pretty_path(path: &Path) -> String {
    let mut joined = path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");

    if path.leading_colon.is_some() {
        joined.push_str("::");
    }

    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_struct() {
        test_derive! {
            heapsizeof_derive {
                struct Foo {
                    a: u32,
                    things: Vec<Foo>,
                    array: [u32; 5],
                }
            }
            expands to {
                #[allow(non_upper_case_globals)]
                const _DERIVE_heapsize_HeapSizeOf_FOR_Foo: () = {
                    extern crate heapsize;

                    impl heapsize::HeapSizeOf for Foo {
                        fn heap_size_of_children(&self) -> usize {
                            let mut sum = 0;
                            match * self {
                                Foo {
                                        a: ref __binding_0,
                                        things : ref __binding_1,
                                        array: ref __binding_2,
                                }
                                => {
                                    { sum += __binding_0.heap_size_of_children(); }
                                    { sum += __binding_1.heap_size_of_children(); }
                                    {
                                         for item in __binding_2.iter() {
                                            sum += item.heap_size_of_children();
                                         }
                                    }
                                    }
                                }
                            sum
                        }
                    }
                };
            }
        }
    }

    #[test]
    fn tuple_struct() {
        test_derive! {
            heapsizeof_derive {
                struct Tuple([Box<u32>; 2], Box<u8>);
            }
            expands to {
            }
        }
    }

    #[test]
    #[should_panic(
        expected = "#[ignore_heap_size_of] should have an explanation"
    )]
    fn all_ignored_fields_require_an_explanation() {
        test_derive! {
            heapsizeof_derive {
                struct Blah {
                    #[ignore_heap_size_of]
                    foo: u32,
                }
            }
            expands to {} no_build
        }
    }
}
