extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use struct_iterable_internal::Iterable;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// The `Iterable` proc macro.
///
/// Deriving this macro for your struct will make it "iterable". An iterable struct allows you to iterate over its fields, returning a tuple containing the field name as a static string and a reference to the field's value as `dyn Any`.
///
/// # Limitations
///
/// - Only structs are supported, not enums or unions.
/// - Only structs with named fields are supported.
///
/// # Usage
///
/// Add the derive attribute (`#[derive(Iterable)]`) above your struct definition.
///
/// ```
/// use struct_iterable::Iterable;
///
/// #[derive(Iterable)]
/// struct MyStruct {
///     field1: i32,
///     field2: String,
/// }
/// ```
///
/// You can now call the `iter` method on instances of your struct to get an iterator over its fields:
///
/// ```
/// let my_instance = MyStruct {
///     field1: 42,
///     field2: "Hello, world!".to_string(),
/// };
///
/// for (field_name, field_value) in my_instance.iter() {
///     println!("{}: {:?}", field_name, field_value);
/// }
/// ```
#[proc_macro_derive(Iterable, attributes(field_name))]
pub fn derive_iterable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;
    let fields = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields_named) => fields_named.named,
            _ => panic!("Only structs with named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let fields_iter = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let mut field_name = field_ident.as_ref().unwrap().to_string();

        for attr in &field.attrs {
            if attr.path.is_ident("field_name") {
                if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                    for nested in meta_list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(nv)) = nested {
                            if nv.path.is_ident("field_name") {
                                if let syn::Lit::Str(lit_str) = nv.lit {
                                    field_name = lit_str.value();
                                }
                            }
                        }
                    }
                }
            }
        }

        quote! {
            (#field_name, &(self.#field_ident) as &dyn std::any::Any)
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
