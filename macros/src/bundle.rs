use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use syn::{DataStruct, DeriveInput, LitInt, Result};
use syn::{Fields, Ident};
use syn::parse::ParseStream;

mod bundle_attrs {
    syn::custom_keyword!(id);
}

enum BundleAttrs {
    Id { value: LitInt },
}

impl syn::parse::Parse for BundleAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(bundle_attrs::id) {
            input.parse::<bundle_attrs::id>()?;
            input.parse::<syn::Token![=]>()?;
            Ok(BundleAttrs::Id {
                value: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

pub(crate) fn process_bundle(input: syn::DeriveInput, en: DataStruct) -> Result<TokenStream> {
    let ident = input.ident;
    let  bundle_attrs = input.attrs.iter().find(|a| a.path.is_ident("bundle"));
    let bundle_attrs = if let Some(bundle_attrs) = bundle_attrs{
        Some(bundle_attrs.parse_args::<BundleAttrs>()?)
    }else{
        None
    };

    let mut components = Vec::new();
    let mut comp_refs = Vec::new();
    match &en.fields {
        Fields::Named(named) => {
            named.named.iter().for_each(|field| {
                components.push(field.ty.clone());
            });
            comp_refs = named.named.iter().map(|field| {
                let ident = field.ident.clone().unwrap();
                let typ = &field.ty;
                //             let position = &mut self.position as *mut Position;
                quote! {
                    let #ident = &mut self.#ident as *mut #typ;
                    components.push(#ident);
                }
            }).collect();
        }
        Fields::Unnamed(not_named) => {
            not_named.unnamed.iter().for_each(|field| {
                components.push(field.ty.clone());
            });
        }
        Fields::Unit => {
            return Err(syn::Error::new(
                ident.span(),
                "Bundle can only be derived from an struct with named fields",
            ));
        }
    }
let id=
    if let Some(bundle_attrs) = bundle_attrs{
        match bundle_attrs{
            BundleAttrs::Id{value}=> {
                let value = value.base10_parse::<u32>()?;
               value

            },
        }
    }else{
        let mut hasher = DefaultHasher::default();
        ident.to_string().hash(&mut hasher);
        hasher.finish() as u32

    };


    let size = components.len();
    Ok(quote! {
        impl dumbledore::component::Bundle for #ident {
            fn into_component_ptrs(self) -> Box<[(dumbledore::archetypes::ComponentInfo, NonNull<u8>)]>
                where Self: Sized {
                let mut components = Vec::with_capacity(#size);

                return components.into_boxed_slice();
            }
            fn component_info() -> Vec<dumbledore::archetypes::ComponentInfo>  where Self: Sized {
                vec![#(dumbledore::archetypes::ComponentInfo::new::<#components>()),*]
            }
            fn archetype_id() -> u32
                where Self: Sized {
               #id
            }
        }

    })
}
