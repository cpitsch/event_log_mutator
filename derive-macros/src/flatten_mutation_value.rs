use proc_macro2::TokenStream;
use syn::{DeriveInput, Variant};

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
                Self::#ident(#(#fields),*) => iproduct!(#(#fields.get_as_vec()),*)
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
                } => iproduct!(#(#fields.get_as_vec()),*).map(|(#(#fields),*)| Self::#ident {
                    #(#fields: MutationValue::Value(#fields)),*
                }
                ).collect()
            }
        }
    }
}

pub fn impl_flatten_mutation_value(ast: DeriveInput) -> proc_macro::TokenStream {
    let ident = ast.ident;

    let variants = match ast.data {
        syn::Data::Enum(e) => e.variants,
        syn::Data::Union(_) => panic!("Unions not supported."),
        syn::Data::Struct(_) => panic!("Structs not supported."),
    };

    let cases: Vec<TokenStream> = variants
        .into_iter()
        .map(enum_variant_to_match_arm)
        .collect();

    quote::quote! {
        impl FlattenMutationValue for #ident {
            fn flatten(self) -> Vec<Self> {
                match self {
                    #(#cases),*
                }
            }
        }
    }
    .into()
}
