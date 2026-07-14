use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(IntoSchemaType)]
pub fn derive_into_schema_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let name_str = name.to_string();

    let expanded = match &input.data {
        Data::Struct(data_struct) => {
            let fields = match &data_struct.fields {
                Fields::Named(fields) => {
                    let field_conversions = fields.named.iter().map(|field| {
                        let field_name = field.ident.as_ref().unwrap();
                        let field_name_str = field_name.to_string();
                        let field_type = &field.ty;

                        quote! {
                            <#field_type as crate::auto_api::object::IntoSchemaType>::into_struct_field(registry, #field_name_str.to_string())
                        }
                    });

                    quote! {
                        vec![
                            #(#field_conversions),*
                        ]
                    }
                }
                Fields::Unnamed(_) => {
                    return syn::Error::new_spanned(
                        name,
                        "IntoSchemaType can only be derived for structs with named fields",
                    )
                    .to_compile_error()
                    .into();
                }
                Fields::Unit => {
                    quote! { vec![] }
                }
            };

            quote! {
                impl crate::auto_api::object::IntoSchemaType for #name {
                    fn into_schema_object(registry: &mut crate::auto_api::SchemaRegistry) -> crate::auto_api::object::SchemaObject {
                        let fields = #fields;

                        crate::auto_api::object::SchemaObject {
                            name: #name_str.to_string(),
                            kind: crate::auto_api::object::SchemaType::Struct(crate::auto_api::object::SchemaStruct { fields }),
                            nullable: false,
                        }
                    }
                }
            }
        }
        Data::Enum(e) => {
            let variants = e
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    Fields::Unit => {
                        let variant_name = &variant.ident;
                        Ok(variant_name.to_string())
                    }
                    Fields::Named(_) | Fields::Unnamed(_) => Err(syn::Error::new_spanned(
                        &variant.ident,
                        "IntoSchemaType can only be derived for enums with unit variants",
                    )
                    .to_compile_error()),
                })
                .collect::<Result<Vec<_>, _>>();

            let variants = match variants {
                Ok(v) => v,
                Err(e) => return e.into(),
            };

            quote! {
                impl crate::auto_api::object::IntoSchemaType for #name {
                    fn into_schema_object(registry: &mut crate::auto_api::SchemaRegistry) -> crate::auto_api::object::SchemaObject {
                        let variants = vec![
                            #(#variants.to_string()),*
                        ];

                        crate::auto_api::object::SchemaObject {
                            name: #name_str.to_string(),
                            kind: crate::auto_api::object::SchemaType::Enum(crate::auto_api::object::SchemaEnum {
                                variants,
                            }),
                            nullable: false,
                        }
                    }
                }
            }
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(name, "IntoSchemaType cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IntoSchema)]
pub fn derive_into_schema(input: TokenStream) -> TokenStream {
    // Alias for IntoSchemaType
    derive_into_schema_type(input)
}
