use deluxe::extract_attributes;
use proc_macro::TokenStream;
use syn::DeriveInput;

use crate::{
    utils::{type_is_bool, type_is_option},
    FieldAttributes,
};

fn named_field_to_quote(field: &mut syn::Field) -> deluxe::Result<proc_macro2::TokenStream> {
    let ident = field.ident.clone().unwrap();
    let name = ident.to_string();
    let attributes: FieldAttributes = extract_attributes(field)?;

    let is_no_space_name = attributes.no_split;
    let name = attributes.rename.unwrap_or(name);

    if type_is_option(&field.ty) {
        Ok(quote::quote! {
            match self.#ident.clone() {
                Some(val) => {
                    format!("{}{}{}", #name, if #name.is_empty() || #is_no_space_name {""} else {"_"}, val)
                }
                None => format!("No_{}", #name)
            }
        })
    } else if type_is_bool(&field.ty) {
        Ok(quote::quote! {
            format!("{}{}", if self.#ident {""} else {"no_"}, #name)
        })
    } else {
        Ok(quote::quote! {
            format!("{}{}{}", #name, if #name.is_empty() || #is_no_space_name {""} else {"_"}, self.#ident)
        })
    }
}

pub fn impl_as_dir_name(ast: DeriveInput) -> TokenStream {
    let ident = ast.ident;
    let ident_str = ident.to_string();

    let mut field_objs: syn::Fields = match ast.data {
        syn::Data::Enum(_) => panic!("Enums not supported"),
        syn::Data::Union(_) => panic!("Unions not supported (yet?)"),
        syn::Data::Struct(data) => data.fields,
    };

    let attr_str_quotes: Vec<proc_macro2::TokenStream> = field_objs
        .iter_mut()
        .map(|field| named_field_to_quote(field).unwrap())
        .collect();

    quote::quote! {
        impl DirName for #ident {
            fn to_dir_name(&self) -> String {
                let mut name_components: Vec<String> = vec![#(#attr_str_quotes),*];
                name_components.insert(0, #ident_str.to_string());
                format!("{}", name_components.join("_"))
            }
        }
    }
    .into()
}
