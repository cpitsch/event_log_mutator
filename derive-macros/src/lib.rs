use deluxe::extract_attributes;
use proc_macro::TokenStream;
use syn::DeriveInput;
use syn::TypePath;

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(asdirname))]
struct FieldAttributes {
    rename: Option<String>,
    #[deluxe(default = false)]
    no_split: bool,
}

// fn extract_attrs_for_field(field: &mut syn::Field) -> deluxe::Result<FieldAttributes> {
//     deluxe::extract_attributes(field)
// }
//
// fn extract_field_attrs(ast: &mut DeriveInput) -> deluxe::Result<HashMap<String, FieldAttributes>> {
//     let mut attrs: HashMap<String, FieldAttributes> = HashMap::new();
//
//     if let syn::Data::Struct(s) = &mut ast.data {
//         for field in s.fields.iter_mut() {
//             let name = field.ident.as_ref().unwrap().to_string();
//             attrs.insert(name, extract_attrs_for_field(field)?);
//         }
//     }
//     Ok(attrs)
// }

#[proc_macro_derive(AsDirName, attributes(asdirname))]
pub fn as_dir_name_macro(item: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(item).unwrap();
    impl_as_dir_name(ast)
}

fn type_matches_str(ty: &syn::Type, type_str: &str) -> bool {
    match ty {
        syn::Type::Path(TypePath { path, .. }) => {
            path.segments.iter().any(|seg| seg.ident == type_str)
        }
        _ => false,
    }
}

fn type_is_option(ty: &syn::Type) -> bool {
    type_matches_str(ty, "Option")
}

fn type_is_bool(ty: &syn::Type) -> bool {
    type_matches_str(ty, "bool")
}

fn field_obj_to_quote(field: &mut syn::Field) -> deluxe::Result<proc_macro2::TokenStream> {
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

fn impl_as_dir_name(ast: DeriveInput) -> TokenStream {
    let ident = ast.ident;
    let ident_str = ident.to_string();

    let mut field_objs: syn::Fields = match ast.data {
        syn::Data::Enum(_) => panic!("Enums not supported"),
        syn::Data::Union(_) => panic!("Unions not supported (yet?)"),
        syn::Data::Struct(data) => data.fields,
    };

    let attr_str_quotes: Vec<proc_macro2::TokenStream> = field_objs
        .iter_mut()
        .map(|field| field_obj_to_quote(field).unwrap())
        .collect();

    quote::quote! {
        impl AsDirName for #ident {
            fn as_dir_name(&self) -> String {
                let attr_strs: Vec<String> = vec![#(#attr_str_quotes),*];
                format!("{}_{}", #ident_str, attr_strs.join("_"))
            }
        }
    }
    .into()
}
