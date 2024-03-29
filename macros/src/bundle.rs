use proc_macro2::TokenStream;
use quote::quote;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use syn::parse::ParseStream;
use syn::Fields;
use syn::{DataStruct, DeriveInput, LitBool, LitInt, Result};

mod bundle_attrs {
    syn::custom_keyword!(id);
    syn::custom_keyword!(generate_lookup);
}

enum BundleAttrs {
    Id { value: LitInt },
    GenerateLookup { value: LitBool },
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
        } else if lookahead.peek(bundle_attrs::generate_lookup) {
            input.parse::<bundle_attrs::id>()?;
            input.parse::<syn::Token![=]>()?;
            Ok(BundleAttrs::GenerateLookup {
                value: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

pub(crate) fn process_bundle(input: DeriveInput, en: DataStruct) -> Result<TokenStream> {
    let ident = input.ident;

    let mut id: Option<u32> = None;
    let mut generate_lookup: Option<bool> = None;

    for attr in input.attrs.iter() {
        if attr.path.is_ident("bundle") {
            let bundle_attrs = attr.parse_args::<BundleAttrs>()?;
            match bundle_attrs {
                BundleAttrs::Id { value } => {
                    id = Some(value.base10_parse()?);
                }
                BundleAttrs::GenerateLookup { value } => {
                    generate_lookup = Some(value.value());
                }
            }
        }
    }

    let mut components = Vec::new();
    let mut comp_refs = Vec::new();
    match &en.fields {
        Fields::Named(named) => {
            named.named.iter().for_each(|field| {
                let ident = field.ident.clone().unwrap();
                let typ = &field.ty;
                components.push(field.ty.clone());

                comp_refs.push(quote! {
                    let #ident = &mut self.#ident as *mut #typ;
                    f(#ident.cast(),dumbledore::archetypes::ComponentInfo::new::<#typ>());
                    std::mem::forget(self.#ident);
                });
            });
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
    let id = id.unwrap_or_else(|| {
        let mut hasher = DefaultHasher::default();
        ident.to_string().hash(&mut hasher);
        hasher.finish() as u32
    });
    if let Some(value) = generate_lookup {
        if value {
            println!("TODO: generate lookup for {}", ident);
            // TODO: generate lookup
        }
    }
    Ok(quote! {
        impl dumbledore::component::Bundle for #ident {
            unsafe fn put_self(mut self, mut f: impl FnMut(*mut u8, dumbledore::archetypes::ComponentInfo)) where Self: Sized {
                #(
                    #comp_refs
                )*
             }
            fn component_info() -> Vec<dumbledore::archetypes::ComponentInfo>  where Self: Sized {
                vec![#(dumbledore::archetypes::ComponentInfo::new::<#components>()),*]
            }
            #[inline(always)]
            fn archetype_id() -> u32
                where Self: Sized {
               #id
            }
        }

    })
}
