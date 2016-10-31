/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate syn;

use proc_macro::TokenStream;

#[proc_macro_derive(HeapSizeOf)]
pub fn derive_heap_size_of(input: TokenStream) -> TokenStream {
    let type_ = syn::parse_macro_input(&input.to_string()).unwrap();
    let name = &type_.ident;
    let (impl_generics, ty_generics, where_clause) = type_.generics.split_for_impl();
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

    tokens.to_string().parse().unwrap()
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
