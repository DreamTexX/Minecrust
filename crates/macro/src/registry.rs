use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result, spanned::Spanned};

#[derive(FromAttributes)]
#[darling(attributes(packet))]
struct PacketArgs {
    id: i32,
    version: i32,
}

pub fn parse_registry(input: DeriveInput) -> TokenStream {
    let match_arms = match parse_registry_data(&input.data) {
        Ok(stream) => stream,
        Err(err) => return err.into_compile_error(),
    };

    let item_name = input.ident;
    quote! {
        impl crate::VersionedDeserialize for #item_name {
            fn deserialize<R: std::io::Read>(protocol_version: i32, reader: &mut R) -> crate::Result<Self> {
                use crate::Deserialize;
                use crate::datatype::VarInt;

                let packet_id = VarInt::deserialize(reader)?;
                Ok(match (*packet_id, protocol_version) {
                    #match_arms
                    (id, version) => return Err(crate::Error::UnknownPacket(id, version)),
                })
            }
        }
    }
}

fn parse_registry_data(data: &Data) -> Result<TokenStream> {
    match data {
        Data::Enum(data) => {
            let mut matrix = vec![];
            for variant in &data.variants {
                let field = match variant.fields {
                    Fields::Unnamed(ref fields) => {
                        if fields.unnamed.len() > 1 {
                            return Err(Error::new(
                                fields.unnamed.span(),
                                "Deserializing more than one packet is not supported",
                            ));
                        }
                        let Some(field) = fields.unnamed.first() else {
                            return Err(Error::new(
                                fields.unnamed.span(),
                                "There must be one packet to deserialize",
                            ));
                        };
                        field
                    }
                    Fields::Unit | Fields::Named(_) => {
                        return Err(Error::new(variant.span(), "This variant is not supported"));
                    }
                };

                let PacketArgs { id, version } = PacketArgs::from_attributes(&variant.attrs)?;
                let packet_ident = &field.ty;
                let variant_ident = &variant.ident;

                matrix.push((
                    version,
                    id,
                    quote! { Self::#variant_ident(#packet_ident::deserialize(reader)?) },
                ));
            }

            matrix.sort_by(|(version_1, ..), (version_2, ..)| version_2.cmp(version_1));

            let match_arms = matrix.iter().map(|(version, id, expr)| {
                quote! {
                    (#id, #version..) => #expr,
                }
            });

            Ok(quote! {
                #(#match_arms)*
            })
        }
        Data::Struct(data) => Err(Error::new(
            data.struct_token.span,
            "Deserializing into structs is not supported",
        )),
        Data::Union(data) => Err(Error::new(
            data.union_token.span,
            "Deserializing into unions is not supported",
        )),
    }
}
