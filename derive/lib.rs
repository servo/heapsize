/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![cfg_attr(not(test), feature(proc_macro, proc_macro_lib))]

#[cfg(not(test))] extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate syn;

#[cfg(not(test))]
#[proc_macro_derive(HeapSizeOf)]
pub fn expand_token_stream(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_string(&input.to_string()).parse().unwrap()
}

fn expand_string(input: &str) -> String {
    let type_ = syn::parse_macro_input(input).unwrap();

    let variant_code = match type_.body {
        syn::Body::Struct(ref data) => {
            vec![expand_variant(type_.ident.clone().into(), data)]
        }
        syn::Body::Enum(ref variants) => {
            variants.iter().map(|variant| {
                let path = syn::Path {
                    global: false,
                    segments: vec![type_.ident.clone().into(), variant.ident.clone().into()],
                };
                expand_variant(path, &variant.data)
            }).collect()
        }
    };

    let name = &type_.ident;
    let (impl_generics, ty_generics, where_clause) = type_.generics.split_for_impl();
    let mut where_clause = where_clause.clone();
    for param in &type_.generics.ty_params {
        where_clause.predicates.push(syn::WherePredicate::BoundPredicate(syn::WhereBoundPredicate {
            bound_lifetimes: Vec::new(),
            bounded_ty: syn::Ty::Path(None, param.ident.clone().into()),
            bounds: vec![syn::TyParamBound::Trait(
                syn::PolyTraitRef {
                    bound_lifetimes: Vec::new(),
                    trait_ref: syn::parse_path("::heapsize::HeapSizeOf").unwrap(),
                },
                syn::TraitBoundModifier::None
            )],
        }))
    }

    let tokens = quote! {
        #type_

        impl #impl_generics ::heapsize::HeapSizeOf for #name #ty_generics #where_clause {
            #[inline]
            #[allow(unused_variables, unused_mut)]
            fn heap_size_of_children(&self) -> usize {
                let mut sum = 0;
                match *self {
                    #( #variant_code )*
                }
                sum
            }
        }
    };

    tokens.to_string()
}

fn expand_variant(path: syn::Path, variant: &syn::VariantData) -> quote::Tokens {
    let mut fields = Vec::new();
    let mut summed_fields = Vec::new();
    for (i, field) in variant.fields().iter().enumerate() {
        let ignore = field.attrs.iter().any(|attr| match attr.value {
            syn::MetaItem::Word(ref ident) |
            syn::MetaItem::List(ref ident, _) if ident == "ignore_heap_size_of" => {
                panic!("#[ignore_heap_size_of] should have an explanation, \
                        e.g. #[ignore_heap_size_of = \"because reasons\"]");
            }
            syn::MetaItem::NameValue(ref ident, _) if ident == "ignore_heap_size_of" => true,
            _ => false
        });

        let ident = field.ident.clone().unwrap_or_else(|| format!("field_{}", i).into());
        if !ignore {
            summed_fields.push(ident.clone())
        }
        fields.push(ident);
    }
    let pattern = match *variant {
        syn::VariantData::Unit => quote!(#path),
        syn::VariantData::Struct(_) => quote!(#path { #( ref #fields ),* }),
        syn::VariantData::Tuple(_) => quote!(#path ( #( ref #fields ),* )),
    };
    quote! {
        #pattern => {
            #(
                sum += ::heapsize::HeapSizeOf::heap_size_of_children(#summed_fields);
            )*
        }
    }
}

#[test]
fn test_struct() {
    let source = "struct Foo<T> { bar: Bar, baz: T, #[ignore_heap_size_of = \"\"] z: Arc<T> }";
    let expanded = expand_string(source);
    macro_rules! contains {
        ($e: expr) => {
            assert!(expanded.replace(" ", "").contains(&$e.replace(" ", "")),
                    "{:?} does not contains {:?} (whitespace-insensitive)", expanded, $e)
        }
    }
    contains!(source);
    contains!("impl<T> ::heapsize::HeapSizeOf for Foo<T> where T: ::heapsize::HeapSizeOf {");
    contains!("sum += ::heapsize::HeapSizeOf::heap_size_of_children(bar);");
    contains!("sum += ::heapsize::HeapSizeOf::heap_size_of_children(baz);");
    assert!(!expanded.replace(" ", "").contains("heap_size_of_children(z)"));
}

#[should_panic(expected = "should have an explanation")]
#[test]
fn test_no_reason() {
    expand_string("struct A { #[ignore_heap_size_of] b: C }");
}
