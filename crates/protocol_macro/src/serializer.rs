use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Index};

use crate::FieldAttributes;

pub fn parse_serialize(input: DeriveInput) -> TokenStream {
    let fn_body = match parse_serialize_data(input.data) {
        Ok(stream) => stream,
        Err(err) => return err.into_compile_error(),
    };

    let item_name = input.ident;
    quote! {
        impl crate::Serialize for #item_name {
            fn serialize<B: bytes::BufMut>(&self, buf: &mut B) {
                use crate::Serialize;
                #fn_body
            }
        }
    }
}

fn parse_serialize_data(data: Data) -> Result<TokenStream, Error> {
    match data {
        Data::Struct(data) => Ok(match data.fields {
            Fields::Named(mut fields) => {
                let statements = fields
                    .named
                    .iter_mut()
                    .map(|field| {
                        let FieldAttributes { with } = deluxe::extract_attributes(field)?;
                        let field_ident = &field.ident;
                        Ok(if let Some(with) = with {
                            quote! {
                                #with::serialize(&self.#field_ident, buf);
                            }
                        } else {
                            quote! {
                                self.#field_ident.serialize(buf);
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                quote! {
                    #(#statements)*
                }
            }
            Fields::Unnamed(mut fields) => {
                let statements = fields
                    .unnamed
                    .iter_mut()
                    .enumerate()
                    .map(|(index, field)| {
                        let FieldAttributes { with } = deluxe::extract_attributes(field)?;
                        let field_index = Index::from(index);
                        Ok(if let Some(with) = with {
                            quote! {
                                #with::serialize(&self.#field_index, buf);
                            }
                        } else {
                            quote! {
                                self.#field_index.serialize(buf);
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
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
