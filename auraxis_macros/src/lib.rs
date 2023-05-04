#![feature(proc_macro_diagnostic)]
#![allow(unused_variables)]
#![allow(dead_code)]

use darling::{ast::Data, util, FromDeriveInput, FromField, FromMeta};
use proc_macro::{Diagnostic, Level, TokenStream};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Ident, Type};

#[derive(Debug, FromField)]
#[darling(attributes(census))]
struct CensusQueryField {
    ident: Option<Ident>,
    ty: Type,
    #[darling(default)]
    main: bool,
}

#[derive(Debug, FromMeta)]
#[darling(default)]
struct CensusQueryUrl(String);

impl Default for CensusQueryUrl {
    fn default() -> Self {
        Self("https://census.daybreakgames.com".to_string())
    }
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(census), forward_attrs(allow, doc, cfg))]
struct CensusQueryArgs {
    ident: syn::Ident,
    attrs: Vec<syn::Attribute>,
    data: Data<util::Ignored, CensusQueryField>,
    #[darling(default)]
    api_url: CensusQueryUrl,
    main: Option<String>,
}

#[proc_macro_derive(Query, attributes(census))]
pub fn derive_census_query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let args = CensusQueryArgs::from_derive_input(&input);

    match args {
        Ok(args) => {
            println!("{:?}", &args);

            match args.data {
                Data::Enum(_) => todo!(),
                Data::Struct(data) => {
                    let main_fields: Vec<&CensusQueryField> =
                        data.fields.iter().filter(|field| field.main).collect();

                    let main = if data.fields.len() == 1 {
                        Some(&data.fields[0])
                    } else if data.fields.is_empty() {
                        Diagnostic::spanned(args.ident.span().unwrap(), Level::Error, "Empty struct not allowed").emit();

                        None
                    } else {
                        match main_fields.len() {
                            0 => {
                                if let Some(main_field_name) = args.main {
                                    let main_field = data.fields.iter().find(|field| {
                                        *field.ident.as_ref().unwrap() == main_field_name
                                    });

                                    if main_field.is_none() {
                                        Diagnostic::spanned(
                                            args.ident.span().unwrap(),
                                            proc_macro::Level::Error,
                                            format!(
                                                "{} is not a valid field identifier",
                                                &main_field_name
                                            ),
                                        )
                                            .emit();
                                    }

                                    main_field
                                } else {
                                    Diagnostic::spanned(
                                        args.ident.span().unwrap(),
                                        proc_macro::Level::Error,
                                        "No main field identifier present",
                                    )
                                        .emit();

                                    None
                                }
                            }
                            1 => Some(main_fields[0]),
                            _ => {
                                for main_field in main_fields {
                                    Diagnostic::spanned(
                                        main_field.ident.span().unwrap(),
                                        proc_macro::Level::Error,
                                        "Multiple main fields specified for query. Please choose one.",
                                    )
                                        .emit()
                                }

                                None
                            }
                        }
                    }.expect("No main field found");

                    let collections: Vec<String> = data
                        .fields
                        .iter()
                        .map(|field| field.ident.as_ref().unwrap().clone().to_string())
                        .collect();

                    let query_name = input.ident;
                    let model = &main.ty;
                    let main_collection = main.ident.as_ref().unwrap().to_string();

                    let expanded = quote! {
                        #[async_trait::async_trait]
                        impl auraxis::api::Query<#model> for #query_name {
                            type Output = #model;

                            async fn execute(client: &auraxis::api::client::ApiClient) -> Result<Vec<Self::Output>, Box<dyn std::error::Error>> {
                                use auraxis::api::CensusModel;

                                let CensusResponse { count, items} = client.get(#model::collection()).build().await?;

                                let items: Vec<#model> = items.into_iter().map(|item| {
                                    let char: #model = serde_json::from_value(item).unwrap();

                                    char
                                }).collect();

                                Ok(items)
                            }
                        }
                    };

                    TokenStream::from(expanded)
                }
            }
        }
        Err(err) => err.write_errors().into(),
    }
}
