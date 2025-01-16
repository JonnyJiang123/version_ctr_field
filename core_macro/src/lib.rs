use syn::{parse_macro_input, DeriveInput};

mod version_derive;

#[proc_macro_derive(VersionCtr, attributes(version))]
pub fn version_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    version_derive::version_derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
