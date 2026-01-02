use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Index};

pub fn parse_serialize(input: DeriveInput) -> TokenStream {
    let fn_body = match parse_serialize_data(&input.data) {
        Ok(stream) => stream,
        Err(err) => return err.into_compile_error().into(),
    };

    let item_name = input.ident;
    quote! {
        impl crate::Serialize for #item_name {
            fn serialize<B: BufMut>(&self, buf: &mut B) {
                use crate::Serialize;
                #fn_body
            }
        }
    }
}

fn parse_serialize_data(data: &Data) -> Result<TokenStream, Error> {
    match data {
        Data::Struct(data) => Ok(match data.fields {
            Fields::Named(ref fields) => {
                let statements = fields.named.iter().map(|field| {
                    let field_ident = &field.ident;
                    quote! {
                        self.#field_ident.serialize(buf);
                    }
                });
                quote! {
                    #(#statements)*
                }
            }
            Fields::Unnamed(ref fields) => {
                let statements = fields.unnamed.iter().enumerate().map(|(index, _)| {
                    let field_index = Index::from(index);
                    quote! {
                        self.#field_index.serialize(buf);
                    }
                });
                quote! {
                    #(#statements)*
                }
            }
            Fields::Unit => quote! {},
        }),
        Data::Enum(data) => Err(Error::new(
            data.enum_token.span,
            "Serializing enums is not supported",
        )),
        Data::Union(data) => Err(Error::new(
            data.union_token.span,
            "Serializing unions is not supported",
        )),
    }
}
