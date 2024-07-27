use syn::TypePath;

pub fn type_matches_str(ty: &syn::Type, type_str: &str) -> bool {
    match ty {
        syn::Type::Path(TypePath { path, .. }) => {
            path.segments.iter().any(|seg| seg.ident == type_str)
        }
        _ => false,
    }
}

pub fn type_is_option(ty: &syn::Type) -> bool {
    type_matches_str(ty, "Option")
}

pub fn type_is_bool(ty: &syn::Type) -> bool {
    type_matches_str(ty, "bool")
}
