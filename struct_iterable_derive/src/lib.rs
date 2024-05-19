extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Ident, Lit, Meta,
    MetaNameValue,
};

#[proc_macro_derive(Iterable, attributes(field_name))]
pub fn derive_iterable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named.named,
            _ => panic!("Only structs with named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let fields_iter = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let default_name = field_ident.as_ref().unwrap().to_string();
        let custom_name = find_custom_name(&field.attrs, "field_name").unwrap_or(default_name);

        quote! {
            (#custom_name, &(self.#field_ident) as &dyn std::any::Any)
        }
    });

    let expanded = quote! {
        impl Iterable for #struct_name {
            fn iter<'a>(&'a self) -> std::vec::IntoIter<(&'static str, &'a dyn std::any::Any)> {
                vec![
                    #(#fields_iter),*
                ].into_iter()
            }
        }
    };

    TokenStream::from(expanded)
}

fn find_custom_name(attrs: &[Attribute], name: &str) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if attr.path().is_ident(name) {
            match attr.parse_args::<MetaNameValue>() {
                Ok(meta_name_value) if meta_name_value.path.is_ident(name) => {
                    if let Expr::Lit(lit) = meta_name_value.value {
                        if let ExprLit {
                            lit: Lit::Str(lit_str),
                            ..
                        } = lit
                        {
                            return Some(lit_str.value());
                        }
                    }
                }
                _ => {}
            }
        }
        None
    })
}
