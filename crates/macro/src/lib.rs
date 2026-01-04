use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod deserializer;
mod serializer;

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    deserializer::parse_deserialize(input).into()
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    serializer::parse_serialize(input).into()
}
