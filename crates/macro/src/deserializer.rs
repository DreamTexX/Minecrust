use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, LitInt, spanned::Spanned};

pub fn parse_deserialize(input: DeriveInput) -> TokenStream {
    let fn_body = match parse_deserialize_data(&input.data) {
        Ok(stream) => stream,
        Err(err) => return err.into_compile_error().into(),
    };

    let item_name = input.ident;
    quote! {
        impl crate::Deserialize for #item_name {
            fn deserialize<R: std::io::Read>(reader: &mut R) -> crate::Result<Self> {
                use crate::Deserialize;
                #fn_body
            }
        }
    }
}

fn parse_deserialize_data(data: &Data) -> Result<TokenStream, Error> {
    match data {
        Data::Struct(data) => Ok(match data.fields {
            Fields::Named(ref fields) => {
                let struct_contents = fields.named.iter().map(|field| {
                    let field_ident = &field.ident;
                    let field_type = &field.ty;
                    quote! {
                        #field_ident: #field_type::deserialize(reader)?,
                    }
                });
                quote! {
                    Ok(Self {
                        #(#struct_contents)*
                    })
                }
            }
            Fields::Unnamed(ref fields) => {
                let struct_contents = fields.unnamed.iter().map(|field| {
                    let field_type = &field.ty;
                    quote! {
                        #field_type::deserialize(reader)?,
                    }
                });
                quote! {
                    Ok(Self(#(#struct_contents)*))
                }
            }
            Fields::Unit => {
                quote! { Ok(Self) }
            }
        }),
        Data::Enum(data) => {
            // Enums are considered to be collections of packets
            let mut match_arms = vec![];
            for variant in &data.variants {
                let field = match variant.fields {
                    Fields::Unnamed(ref fields) => {
                        if fields.unnamed.len() > 1 {
                            return Err(Error::new(fields.unnamed.span(), "Deserializing more than one packet is not supported"));
                        }
                        let Some(field) = fields.unnamed.first() else {
                            return Err(Error::new(fields.unnamed.span(), "There must be one packet to deserialize"));
                        };
                        field
                    },
                    Fields::Unit | Fields::Named(_) => return Err(Error::new(variant.span(), "This variant is not supported")),
                };

                let Some(attr) = variant.attrs.iter().find(|attr| attr.path().is_ident("id")) else {
                    return Err(Error::new(variant.span(), "This variant requires an id attribute"));
                };
                let id: LitInt = attr.parse_args()?;
                let id = id.base10_parse::<i32>()?;
                let packet_ident = &field.ty;
                let variant_ident = &variant.ident;

                match_arms.push(quote! {
                    #id => Self::#variant_ident(#packet_ident::deserialize(reader)?),
                });  
            };

            Ok(quote! {
                let packet_id = VarInt::deserialize(reader)?;
                Ok(match *packet_id {
                    #(#match_arms)*
                    id => return Err(crate::Error::UnknownPacket(id)),
                })
            })
        },
        Data::Union(data) => Err(Error::new(
            data.union_token.span,
            "Deserializing into unions is not supported",
        )),
    }
}
