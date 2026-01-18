use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields};

use crate::FieldAttributes;

pub fn parse_deserialize(input: DeriveInput) -> TokenStream {
    let fn_body = match parse_deserialize_data(input.data) {
        Ok(stream) => stream,
        Err(err) => return err.into_compile_error(),
    };

    let item_name = input.ident;
    quote! {
        impl crate::Deserialize for #item_name {
            fn deserialize<B: bytes::Buf>(buf: &mut B) -> Result<Self, crate::Error> {
                use crate::Deserialize;
                #fn_body
            }
        }
    }
}

fn parse_deserialize_data(data: Data) -> Result<TokenStream, Error> {
    match data {
        Data::Struct(data) => Ok(match data.fields {
            Fields::Named(mut fields) => {
                let struct_contents = fields
                    .named
                    .iter_mut()
                    .map(|field| {
                        let FieldAttributes { with } = deluxe::extract_attributes(field)?;
                        let field_ident = &field.ident;

                        Ok(if let Some(with) = with {
                            quote! {
                                #field_ident: #with::deserialize(buf)?,
                            }
                        } else {
                            let field_type = &field.ty;
                            quote! {
                                #field_ident: <#field_type>::deserialize(buf)?,
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                quote! {
                    Ok(Self {
                        #(#struct_contents)*
                    })
                }
            }
            Fields::Unnamed(mut fields) => {
                let struct_contents = fields
                    .unnamed
                    .iter_mut()
                    .map(|field| {
                        let FieldAttributes { with } = deluxe::extract_attributes(field)?;
                        Ok(if let Some(with) = with {
                            quote! {
                                #with::deserialize(buf)?,
                            }
                        } else {
                            let field_type = &field.ty;
                            quote! {
                                #field_type::deserialize(buf)?,
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                quote! {
                    Ok(Self(#(#struct_contents)*))
                }
            }
            Fields::Unit => {
                quote! { Ok(Self) }
            }
        }),
        Data::Enum(data) => Err(Error::new(
            data.enum_token.span,
            "Deserializing enums is not supported",
        )),
        Data::Union(data) => Err(Error::new(
            data.union_token.span,
            "Deserializing unions is not supported",
        )),
    }
}
