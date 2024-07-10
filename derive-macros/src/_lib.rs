use proc_macro::TokenStream;
use syn::DeriveInput;
use syn::Ident;
use syn::TypePath;

#[proc_macro_derive(AsDirName)]
pub fn as_dir_name_macro(item: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(item).unwrap();
    impl_as_dir_name(ast)
}

fn impl_as_dir_name(ast: DeriveInput) -> TokenStream {
    let ident = ast.ident;
    let ident_str = ident.to_string();

    let field_objs: syn::Fields = match ast.data {
        syn::Data::Enum(_) => panic!("Enums not supported"),
        syn::Data::Union(_) => panic!("Unions not supported (yet?)"),
        syn::Data::Struct(data) => data.fields,
    };

    let fields: Vec<Ident> = field_objs.iter().filter_map(|f| f.ident.clone()).collect();
    let fields_strings: Vec<String> = fields.iter().map(|f| f.to_string()).collect();
    // let fields_are_options: Vec<bool> = field_objs
    //     .iter()
    //     .map(|f| match f.ty.clone() {
    //         syn::Type::Path(TypePath { path, .. }) => {
    //             path.segments.iter().any(|seg| seg.ident == "Option")
    //         }
    //         _ => false,
    //     })
    //     .collect();

    println!("{}", ident_str);
    let fields_are_options: Vec<bool> = field_objs
        .iter()
        .map(|f| match &f.ty {
            syn::Type::Path(TypePath { path, .. }) => {
                let is_option = path.segments.iter().any(|seg| seg.ident == "Option");
                println!(
                    "\t{} is an option: {}",
                    &f.ident.clone().unwrap(),
                    is_option
                );
                is_option
            }
            _ => false,
        })
        .collect();

    quote::quote! {
        impl AsDirName for #ident {
            fn as_dir_name(&self) -> String {
                let attr_strs: Vec<String> = vec![
                    #(
                        if #fields_are_options {
                            println!("{} is an option", #fields_strings);
                            format!("{}_{}", #fields_strings, self.#fields)
                            // match self.#fields {
                            //     Some(val) => format!("{}_{}", #fields_strings, val),
                            //     None => format!("No_{}", #fields_strings)
                            // }
                        } else {
                            println!("{} is NO option", #fields_strings);
                            format!("{}_{}", #fields_strings, self.#fields)
                        }
                    ),*
                    // #(format!("{}_{}", #fields_strings, self.#fields)),*
                ];
                format!("{}_{}", #ident_str, attr_strs.join("_"))
            }
        }
    }
    .into()
}
