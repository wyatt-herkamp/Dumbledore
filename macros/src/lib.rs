use quote::quote;

mod bundle;

#[proc_macro_derive(Bundle, attributes(bundle))]
pub fn bundle(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(stream as syn::DeriveInput);
    match input.data {
        syn::Data::Struct(ref en) => {
            let data_struct = en.clone();
            bundle::process_bundle(input, data_struct)
                .unwrap_or_else(|e| e.to_compile_error())
                .into()
        }
        _ => panic!("Bundle can only be derived from an struct"),
    }
}

#[proc_macro_derive(Component)]
pub fn component(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse_macro_input!(stream as syn::DeriveInput);
    let ident = input.ident;
    (quote! {
        impl Component for #ident {}
    }).into()
}
