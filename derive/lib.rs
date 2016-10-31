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
                    trait_ref: syn::Path {
                        global: true,
                        segments: vec!["heapsize".into(), "HeapSizeOf".into()],
                    }
                },
                syn::TraitBoundModifier::None
            )],
        }))
    }

    let tokens = quote! {
        #type_

        impl #impl_generics ::heapsize::HeapSizeOf for #name #ty_generics #where_clause {
            #[inline]
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
    let mut open = quote::Tokens::new();
    let mut close = quote::Tokens::new();
    let fields = match *variant {
        syn::VariantData::Unit => return quote!(#path => {}),
        syn::VariantData::Struct(ref fields) => {
            open.append("{");
            close.append("}");
            fields
        }
        syn::VariantData::Tuple(ref fields) => {
            open.append("(");
            close.append(")");
            fields
        }
    };
    let field: Vec<syn::Ident> = fields.iter().enumerate().map(|(i, field)| {
        field.ident.clone().unwrap_or_else(|| format!("field_{}", i).into())
    }).collect();
    let field = &field;
    quote! {
        #path #open #( ref #field ),* #close => {
            #(
                sum += ::heapsize::HeapSizeOf::heap_size_of_children(#field);
            )*
        }
    }
}

#[test]
fn test_struct() {
    let source = "struct Foo<T> { bar: Bar, baz: T }";
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
}
