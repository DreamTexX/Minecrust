use deluxe::ExtractAttributes;
use proc_macro::TokenStream;
use syn::{DeriveInput, Path, parse_macro_input};

mod deserializer;
mod serializer;

#[derive(Debug, ExtractAttributes)]
#[deluxe(attributes(protocol))]
struct FieldAttributes {
    with: Option<Path>,
}

#[proc_macro_derive(Deserialize, attributes(protocol))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    deserializer::parse_deserialize(input).into()
}

#[proc_macro_derive(Serialize, attributes(protocol))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    serializer::parse_serialize(input).into()
}
