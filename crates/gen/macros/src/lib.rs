extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::iter::FromIterator;

#[proc_macro]
pub fn table(name: TokenStream) -> TokenStream {
    let ident = syn::parse_macro_input!(name as syn::Ident);

    quote!(
        #[derive(Copy, Clone)]
        pub struct #ident {
            pub reader: &'static super::TypeReader,
            pub row: super::Row,
        }

        impl PartialEq for #ident {
            fn eq(&self, other: &Self) -> bool {
                self.row == other.row
            }
        }

        impl Eq for #ident {}

        impl Ord for #ident {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.row.cmp(&other.row)
            }
        }

        impl PartialOrd for #ident {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn type_code(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let input = syn::parse_macro_input!(input as syn::ItemEnum);

    if args.len() != 1 {
        panic!("type_code");
    }

    let bits = &args[0];
    let name = &input.ident;
    let mut variants = Vec::new();
    let mut decodes = Vec::new();
    let mut encodes = Vec::new();
    let mut enumerator = 0;

    for variant in input.variants.iter() {
        let name = &variant.ident;
        let table = format_ident!("{}", name);

        if let Some((_, syn::Expr::Lit(value))) = &variant.discriminant {
            if let syn::Lit::Int(value) = &value.lit {
                enumerator = value.base10_parse::<u32>().unwrap();
            }
        }

        variants.push(quote!(
            #name(tables::#name),
        ));

        decodes.push(quote!(
            #enumerator => Self::#name( tables::#name{ reader, row:Row::new(code.1, TableIndex::#table, file) }),
        ));

        encodes.push(quote!(
            Self::#name(value) => ((value.row.index + 1) << #bits) | #enumerator,
        ));

        enumerator += 1;
    }

    let variants = TokenStream2::from_iter(variants);
    let decodes = TokenStream2::from_iter(decodes);
    let encodes = TokenStream2::from_iter(encodes);

    let output = quote!(
        #[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
        pub enum #name {
            #variants
        }
        impl Decode for #name {
            fn decode(reader: &'static TypeReader, code: u32, file:u16) -> Self {
                let code = (code & ((1 << #bits) - 1), (code >> #bits) - 1);
                match code.0 {
                    #decodes
                    _ => panic!("type_code"),
                }
            }
        }
        impl #name {
            pub fn encode(&self) -> u32 {
                match self {
                    #encodes
                }
            }
        }
    );

    output.into()
}
