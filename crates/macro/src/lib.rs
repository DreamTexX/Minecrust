use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Index, parse_macro_input};

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fn_body = match parse_deserialize_data(&input.data) {
        Ok(stream) => stream,
        Err(err) => return err.into_compile_error().into(),
    };

    let item_name = input.ident;
    let expanded = quote! {
        impl crate::Deserialize for #item_name {
            fn deserialize<R: std::io::Read>(reader: &mut R) -> crate::Result<Self> {
                use crate::Deserialize;
                #fn_body
            }
        }
    };
    TokenStream::from(expanded)
}

fn parse_deserialize_data(data: &Data) -> Result<proc_macro2::TokenStream, Error> {
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
        Data::Enum(data) => Err(Error::new(
            data.enum_token.span,
            "Deserializing into enums is not supported",
        )),
        Data::Union(data) => Err(Error::new(
            data.union_token.span,
            "Deserializing into unions is not supported",
        )),
    }
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let fn_body = match parse_serialize_data(&input.data) {
        Ok(stream) => stream,
        Err(err) => return err.into_compile_error().into(),
    };

    let item_name = input.ident;
    let expanded = quote! {
        impl crate::Serialize for #item_name {
            fn serialize<B: BufMut>(&self, buf: &mut B) {
                use crate::Serialize;
                #fn_body
            }
        }
    };
    TokenStream::from(expanded)
}

fn parse_serialize_data(data: &Data) -> Result<proc_macro2::TokenStream, Error> {
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
