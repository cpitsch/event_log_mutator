use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput, Variant};

fn enum_variant_to_match_arm(variant: Variant) -> TokenStream {
    let ident = variant.ident;
    let fields = variant.fields;

    match fields {
        syn::Fields::Unit => quote::quote! {
            Self::#ident => vec![Self::#ident]
        },
        syn::Fields::Unnamed(unnamed_fields) => {
            let fields: Vec<_> = unnamed_fields
                .unnamed
                .into_iter()
                .enumerate()
                .map(|(i, _)| {
                    syn::Ident::new(
                        format!("field{}", i).as_str(),
                        proc_macro2::Span::call_site(),
                    )
                })
                .collect();
            quote::quote! {
                Self::#ident(#(#fields),*) => itertools::iproduct!(#(#fields.get_as_vec()),*)
                    .map(|(#(#fields),*)| Self::#ident(#(MutationValue::Value(#fields)),*))
                    .collect()
            }
        }
        syn::Fields::Named(named_fields) => {
            let fields: Vec<_> = named_fields
                .named
                .into_iter()
                .map(|field| field.ident.unwrap())
                .collect();
            quote::quote! {
                Self::#ident {
                    #(#fields),*
                } => itertools::iproduct!(#(#fields.get_as_vec()),*).map(|(#(#fields),*)| Self::#ident {
                    #(#fields: MutationValue::Value(#fields)),*
                }
                ).collect()
            }
        }
    }
}

fn struct_to_flatten_function_content(s: DataStruct) -> TokenStream {
    let fields: Vec<_> = s
        .fields
        .into_iter()
        .map(|field| field.ident.unwrap())
        .collect();
    quote! {
        itertools::iproduct!(#(self.#fields.get_as_vec()),*).map(|(#(#fields),*)| Self {
            #(#fields: MutationValue::Value(#fields)),*
        }).collect()
    }
}

pub fn impl_flatten_mutation_value(ast: DeriveInput) -> proc_macro::TokenStream {
    let ident = ast.ident;

    let flatten_function_content = match ast.data {
        syn::Data::Enum(e) => {
            let cases = e.variants.into_iter().map(enum_variant_to_match_arm);
            quote::quote! {
                match self {
                    #(#cases),*
                }
            }
        }
        syn::Data::Struct(s) => struct_to_flatten_function_content(s),
        syn::Data::Union(_) => panic!("Unions not supported"),
    };

    quote::quote! {
        impl FlattenMutationValue for #ident {
            fn flatten(self) -> Vec<Self> {
                #flatten_function_content
            }
        }
    }
    .into()
}
