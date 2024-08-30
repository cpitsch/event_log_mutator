use core::panic;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataStruct, DeriveInput, GenericArgument, PathArguments, Variant};

fn is_mutation_value(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if type_path.path.segments.len() == 1 {
            return &type_path.path.segments[0].ident == "MutationValue";
        }
    }
    false
}

fn is_optional_mutation_value(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if type_path.path.segments.len() == 1 {
            let segment = &type_path.path.segments[0];

            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(angle_bracketed) = &segment.arguments {
                    if angle_bracketed.args.len() == 1 {
                        if let GenericArgument::Type(ty_) = &angle_bracketed.args[0] {
                            return is_mutation_value(ty_);
                        }
                    }
                }
            }
        }
    }
    false
}

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
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    syn::Ident::new(
                        format!("field{}", i).as_str(),
                        proc_macro2::Span::call_site(),
                    )
                })
                .collect();

            let mutation_value_fields: Vec<_> = unnamed_fields
                .unnamed
                .iter()
                .enumerate()
                .filter(|field| is_mutation_value(&field.1.ty))
                .map(|(i, _)| {
                    syn::Ident::new(
                        format!("field{}", i).as_str(),
                        proc_macro2::Span::call_site(),
                    )
                })
                .collect();
            let optional_mutation_value_fields: Vec<_> = unnamed_fields
                .unnamed
                .iter()
                .enumerate()
                .filter(|field| is_optional_mutation_value(&field.1.ty))
                .map(|(i, _)| {
                    syn::Ident::new(
                        format!("field{}", i).as_str(),
                        proc_macro2::Span::call_site(),
                    )
                })
                .collect();
            let other_fields: Vec<_> = unnamed_fields
                .unnamed
                .iter()
                .enumerate()
                .filter(|field| {
                    !is_mutation_value(&field.1.ty) && !is_optional_mutation_value(&field.1.ty)
                })
                .map(|(i, _)| {
                    syn::Ident::new(
                        format!("field{}", i).as_str(),
                        proc_macro2::Span::call_site(),
                    )
                })
                .collect();

            let all_iproduct_quotes: Vec<_> = mutation_value_fields
                .iter()
                .map(|ident| {
                    quote::quote! {
                        #ident.get_as_vec()
                    }
                })
                .chain(optional_mutation_value_fields.iter().map(|ident| {
                    quote::quote! {
                        #ident.map_or(vec![None], |mutation_val| {
                            mutation_val
                                .get_as_vec()
                                .into_iter()
                                .map(|val| Some(MutationValue::Value(val))).collect()
                        })
                    }
                }))
                .collect();

            let all_mutation_value_idents = [
                mutation_value_fields.clone(),
                optional_mutation_value_fields.clone(),
            ]
            .concat();

            let all_field_quotes: Vec<_> = mutation_value_fields
                .iter()
                .map(|ident| {
                    quote::quote! {
                        MutationValue::Value(#ident)
                    }
                })
                .chain(optional_mutation_value_fields.iter().map(|ident| {
                    quote::quote! {
                        #ident
                    }
                }))
                .chain(other_fields.iter().map(|ident| {
                    quote::quote! {
                        #ident
                    }
                }))
                .collect();

            quote::quote! {
                Self::#ident(#(#fields),*) => itertools::iproduct!(#(#all_iproduct_quotes),*)
                    .map(|(#(#all_mutation_value_idents),*)| Self::#ident(
                            #(#all_field_quotes),*
                    ))
                    .collect()
            }
        }
        syn::Fields::Named(named_fields) => {
            let fields: Vec<_> = named_fields
                .named
                .iter()
                .cloned()
                .map(|field| field.ident.unwrap())
                .collect();

            let mutation_value_fields: Vec<_> = named_fields
                .named
                .iter()
                .filter(|field| is_mutation_value(&field.ty))
                .cloned()
                .map(|field| field.ident.unwrap())
                .collect();

            let optional_mutation_value_fields: Vec<_> = named_fields
                .named
                .iter()
                .filter(|field| is_optional_mutation_value(&field.ty))
                .cloned()
                .map(|field| field.ident.unwrap())
                .collect();

            let other_fields: Vec<_> = named_fields
                .named
                .iter()
                .filter(|field| {
                    !is_mutation_value(&field.ty) && !is_optional_mutation_value(&field.ty)
                })
                .cloned()
                .map(|field| field.ident.unwrap())
                .collect();

            let all_iproduct_quotes: Vec<_> = mutation_value_fields
                .iter()
                .map(|ident| {
                    quote::quote! {
                        #ident.get_as_vec()
                    }
                })
                .chain(optional_mutation_value_fields.iter().map(|ident| {
                    quote::quote! {
                        #ident.map_or(vec![None], |mutation_val| {
                            mutation_val
                                .get_as_vec()
                                .into_iter()
                                .map(|val| Some(MutationValue::Value(val))).collect()
                        })
                    }
                }))
                .collect();

            let all_mutation_value_idents = [
                mutation_value_fields.clone(),
                optional_mutation_value_fields.clone(),
            ]
            .concat();

            let all_field_quotes: Vec<_> = mutation_value_fields
                .iter()
                .map(|ident| {
                    quote::quote! {
                        #ident: MutationValue::Value(#ident)
                    }
                })
                .chain(optional_mutation_value_fields.iter().map(|ident| {
                    quote::quote! {
                        #ident
                    }
                }))
                .chain(other_fields.iter().map(|ident| {
                    quote::quote! {
                        #ident: #ident.clone()
                    }
                }))
                .collect();

            quote::quote! {
                Self::#ident {
                    #(#fields),*
                } => itertools::iproduct!(#(#all_iproduct_quotes),*)
                    .map(|(#(#all_mutation_value_idents),*)| Self::#ident {
                        #(#all_field_quotes),*
                    }).collect()
            }
        }
    }
}

fn struct_to_flatten_function_content(s: DataStruct) -> TokenStream {
    // Get fields of type MutationValue<_>
    let mutation_value_fields: Vec<_> = s
        .fields
        .iter()
        .filter(|field| is_mutation_value(&field.ty))
        .cloned()
        .map(|field| field.ident.unwrap())
        .collect();

    // Get fields of type Option<MutationValue<_>>
    let optional_mutation_value_fields: Vec<_> = s
        .fields
        .iter()
        .filter(|field| is_optional_mutation_value(&field.ty))
        .cloned()
        .map(|field| field.ident.unwrap())
        .collect();

    // All other fields
    let other_fields: Vec<_> = s
        .fields
        .iter()
        .filter(|field| !is_mutation_value(&field.ty) && !is_optional_mutation_value(&field.ty))
        .cloned()
        .map(|field| field.ident.unwrap())
        .collect();

    let all_iproduct_quotes: Vec<_> = mutation_value_fields
        .iter()
        .map(|ident| {
            quote::quote! {
                self.#ident.get_as_vec()
            }
        })
        .chain(optional_mutation_value_fields.iter().map(|ident| {
            quote::quote! {
                self.#ident.map_or(vec![None], |mutation_val| {
                    mutation_val
                        .get_as_vec()
                        .into_iter()
                        .map(|val| Some(MutationValue::Value(val))).collect()
                })
            }
        }))
        .collect();

    // Need to precompute to ensure correct number of commas when one of the two
    // is empty
    let all_mutation_value_idents = [
        mutation_value_fields.clone(),
        optional_mutation_value_fields.clone(),
    ]
    .concat();

    let all_field_quotes: Vec<_> = mutation_value_fields
        .iter()
        .map(|ident| {
            quote::quote! {
                #ident: MutationValue::Value(#ident)
            }
        })
        .chain(optional_mutation_value_fields.iter().map(|ident| {
            quote::quote! {
                #ident
            }
        }))
        .chain(other_fields.iter().map(|ident| {
            quote::quote! {
                #ident: self.#ident
            }
        }))
        .collect();
    quote! {
        itertools::iproduct!(#(#all_iproduct_quotes),*).map(|(#(#all_mutation_value_idents),*)| Self {
            #(#all_field_quotes),*
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
