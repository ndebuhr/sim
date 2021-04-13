extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SerializableModel)]
pub fn model(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = input.ident;
    let tokens = quote! {
        impl #name {
            pub fn from_value(value: serde_yaml::Value) -> Option<Box<dyn AsModel>> {
                match serde_yaml::from_value::<Self>(value) {
                    Ok(model) => Some(Box::new(model)),
                    Err(_) => None
                }
            }
        }
        impl SerializableModel for #name {
            fn get_type(&self) -> &'static str {
                stringify!(#name)
            }
            fn serialize(&self) -> serde_yaml::Value {
                serde_yaml::to_value(self).unwrap_or(serde_yaml::Value::Null)
            }
        }
    };
    tokens.into()
}
