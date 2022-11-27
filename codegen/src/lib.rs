
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, DeriveInput};
#[proc_macro_derive(PostApi)]
pub fn post_api(input: TokenStream) -> TokenStream {
    let ast:DeriveInput = syn::parse(input).unwrap();
    let ident=ast.ident;
    let gen =quote!(
        impl PostApi for #ident{
            fn post(&self)->serde_json::Value{
                tokio::runtime::Runtime::new().unwrap().block_on(post_reqwest(&self))
            }
        }
    );
    gen.into()

    
}
