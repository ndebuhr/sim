extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemEnum};

#[proc_macro_derive(AsModel)]
pub fn as_model_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let output: proc_macro2::TokenStream = derive_impl(input);

    proc_macro::TokenStream::from(output)
}

fn derive_impl(input: ItemEnum) -> TokenStream {
    let name = input.ident;
    // Tuples of (enum_variant_name, enum_variant_field).
    let variants: Vec<_> = input.variants.into_iter().map(|v| v.ident).collect();

    quote! {
        impl AsModel for #name {
            fn status(&self) -> String {
                match self {
                    #(
                        #name::#variants(item) => item.status(),
                    )*
                }
            }

            fn events_ext(
                &mut self,
                incoming_message: ModelMessage,
                services: &mut Services,
            ) -> Result<Vec<ModelMessage>, SimulationError> {
                match self {
                    #(
                        #name::#variants(item) => item.events_ext(incoming_message, services),
                    )*
                }
            }

            fn events_int(&mut self, services: &mut Services)
                          -> Result<Vec<ModelMessage>, SimulationError> {
                match self {
                    #(
                        #name::#variants(item) => item.events_int(services),
                    )*
                }
            }

            fn time_advance(&mut self, time_delta: f64) {
                match self {
                    #(
                        #name::#variants(item) => item.time_advance(time_delta),
                    )*
                }
            }

            fn until_next_event(&self) -> f64 {
                match self {
                    #(
                        #name::#variants(item) => item.until_next_event(),
                    )*
                }
            }
        }
    }
}
