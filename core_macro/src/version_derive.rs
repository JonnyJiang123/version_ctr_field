use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parse, parse_quote, punctuated::Punctuated, Data, DataStruct, DeriveInput, Fields,
    LitStr, Token,
};

mod kw {
    use syn::custom_keyword;
    custom_keyword!(since);
    custom_keyword!(until);
}

#[derive(Default)]
struct VersionMeta {
    since: Option<LitStr>,
    until: Option<LitStr>,
}
impl VersionMeta {
    fn merge(self, other: Self) -> syn::Result<Self> {
        fn either<T: ToTokens>(a: Option<T>, b: Option<T>) -> syn::Result<Option<T>> {
            match (a, b) {
                (None, None) => Ok(None),
                (Some(val), None) | (None, Some(val)) => Ok(Some(val)),
                (Some(a), Some(b)) => {
                    let mut error = syn::Error::new_spanned(a, "redundant attribute argument");
                    error.combine(syn::Error::new_spanned(b, "note: first one here"));
                    Err(error)
                }
            }
        }
        Ok(Self {
            since: either(self.since, other.since)?,
            until: either(self.until, other.until)?,
        })
    }
}
impl Parse for VersionMeta {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut vm = VersionMeta {
            since: None,
            until: None,
        };
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::since) {
            let _ = input.parse::<kw::since>()?;
            let _ = input.parse::<Token![=]>()?;
            let since: LitStr = input.parse()?;
            vm.since = Some(since);
        } else if lookahead.peek(kw::until) {
            let _ = input.parse::<kw::until>()?;
            let _ = input.parse::<Token![=]>()?;
            let until: LitStr = input.parse()?;
            vm.until = Some(until);
        }
        Ok(vm)
    }
}

pub fn version_derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };
    let struct_name = input.ident;
    let code_tuple_list = fields
        .into_iter()
        .map(|field| {
            let meta = field
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("version"))
                .try_fold(VersionMeta::default(), |meta, attr| {
                    let list: Punctuated<VersionMeta, Token![,]> =
                        attr.parse_args_with(Punctuated::parse_terminated)?;
                    list.into_iter().try_fold(meta, VersionMeta::merge)
                })?;
            let since = meta.since.unwrap_or_else(|| parse_quote!(1f64));
            let until = meta.until.unwrap_or_else(|| parse_quote!(1f64));

            let field_name = field.ident.unwrap();

            let new_name_token_max = format_ident!("{}_version_max", field_name);
            let new_name_token_min = format_ident!("{}_version_min", field_name);

            let version_fn = quote! {
                fn #new_name_token_max() -> f32{
                    #until.parse::<f32>().unwrap()
                }
                fn #new_name_token_min() -> f32{
                    #since.parse::<f32>().unwrap()
                }
            };
            let span = field_name.span();
            let field_name_key = LitStr::new(&field_name.to_string(),span);
            let to_json_code = quote! {
                if Self::#new_name_token_max() >= version && version >= Self::#new_name_token_min() {
                    map.insert(#field_name_key.to_string(),self.#field_name.to_string());
                }
            };
            Ok((version_fn, to_json_code))
        })
        .collect::<Vec<syn::Result<(TokenStream, TokenStream)>>>();

    let mut new_ast = quote!();

    let mut version_func_ast_list: Vec<TokenStream> = vec![];
    let mut to_json_code_list: Vec<TokenStream> = vec![];

    code_tuple_list.into_iter().for_each(|code_tuple| {
        if code_tuple.is_ok() {
            let (version_fn, to_json_code) = code_tuple.unwrap();
            version_func_ast_list.push(version_fn);
            to_json_code_list.push(to_json_code);
        }
    });
    let version_field_ast = quote! {
        impl #struct_name {
            #(#version_func_ast_list)*
        }
    };
    new_ast.extend(version_field_ast);

    // // 实现ToJson
    let to_json_ast = quote! {
        use std::collections::BTreeMap;
        use into_json::IntoJson;
        impl IntoJson for #struct_name {
            fn into_json(self,version: f32) -> String {
                let mut map:BTreeMap<String,String> = BTreeMap::new();
                #(#to_json_code_list)*
                format!("{:?}",map)
            }
        }
        // use std::fmt::{write, Display, Formatter};
        // use std::collections::BTreeMap;
        // impl Display for #struct_name {
        //     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        //         let mut map = BTreeMap::new();
        //         let version = 1.0_f32;
        //          #(#to_json_code_list)*
        //         write!(f,"{:?}",map)
        //     }
        // }
    };
    new_ast.extend(to_json_ast);
    Ok(new_ast)
}
