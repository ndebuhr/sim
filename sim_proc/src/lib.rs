extern crate proc_macro;

use proc_macro2::TokenStream;
use syn::{parse_macro_input, ItemEnum};

#[proc_macro_derive(AsModel)]
pub fn as_model_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as ItemEnum);

    let output: proc_macro2::TokenStream = derive(input);

    proc_macro::TokenStream::from(output)
}

fn derive(_input: ItemEnum) -> TokenStream {
    todo!()
}
