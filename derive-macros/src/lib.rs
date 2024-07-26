use as_dir_name::impl_as_dir_name;
use proc_macro::TokenStream;
use syn::DeriveInput;

mod as_dir_name;
mod utils;

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(dirname))]
struct FieldAttributes {
    rename: Option<String>,
    #[deluxe(default = false)]
    no_split: bool,
}

#[proc_macro_derive(DirName, attributes(dirname))]
pub fn as_dir_name_macro(item: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(item).unwrap();
    impl_as_dir_name(ast)
}
