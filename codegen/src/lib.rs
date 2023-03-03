extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, DeriveInput};
#[proc_macro_derive(PostApi)]
pub fn post_api(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = ast.ident;
    let gen = quote!(
        //#[async_trait::async_trait]
        impl PostApi for #ident{
             fn post(&self)->serde_json::Value{
                //tokio::runtime::Runtime::new().unwrap().block_on(post_reqwest(&self));
               // tokio::runtime::Runtime::new().unwrap().block_on(post_reqwest(&self))
                todo!()
            }
        }
    );
    gen.into()
}
#[proc_macro_derive(ApiName)]
pub fn api_name(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = ast.ident;
    let name = ident.to_string();
    //change name to underline case
    let mut name = name.chars().collect::<Vec<char>>();
    let mut i = 0;
    while i < name.len() {
        if name[i].is_uppercase() {
            name.insert(i, '_');
            i += 1;
        }
        i += 1;
    }

    let name = name.into_iter().collect::<String>().to_lowercase();
    //if contains 'message' replace all 'message' to 'msg'
    let name = name.replace("message", "msg");
    //delete first '_'
    let name = name.trim_start_matches('_').to_string();
    let gen = quote!(
        impl ApiName for #ident{
            fn name(&self)->String{
                #name.to_string()
            }

        }
    );
    gen.into()
}
#[proc_macro_derive(Meassages)]
pub fn meassages(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = ast.ident;
    let gen = quote!(
        impl Meassages for #ident{
            fn start_with(&self, s: &str) -> bool {
                self.message.starts_with(s)
            }
            fn eq(&self, s: &str) -> bool {
                self.message == s
            }
            fn msg(&self) -> &str {
                &self.message
            }
        }
    );
    gen.into()
}
#[proc_macro_attribute]
pub fn handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::ItemFn = syn::parse(item).unwrap();
    if ast.sig.asyncness.is_none() {
        panic!("event must be async function");
    }
    let ident = ast.sig.ident.clone();

    let block = ast.block;
    let gen = quote!(

        #[allow(non_camel_case_types)]
        #[derive(Clone)]
        pub struct #ident;

        #[allow(unused)]
        #[rustqq::async_trait::async_trait]
        impl ::rustqq::app::app::EventHandle for #ident{
            async fn register(&self,event:&Event,data: &::rustqq::app::Config)->Result<(),Box<dyn std::error::Error>>{
                
                #block
            }
        }
    );
    gen.into()
}
#[proc_macro_attribute]
pub fn task(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::ItemFn = syn::parse(item).unwrap();
    let attr: syn::LitStr = syn::parse(attr).unwrap();
    let ident = ast.sig.ident.clone();
    let block = ast.block;
    let gen = quote!(

        #[allow(non_camel_case_types)]
        #[derive(Clone)]
        pub struct #ident;

        #[allow(unused)]
        #[rustqq::async_trait::async_trait]
        impl ::rustqq::app::app::TaskHandle for #ident{
            async fn tasks(&self)->Result<(),Box<dyn std::error::Error>>{
                #block
            }
            fn schedule(&self)->String{
                #attr.to_string()
            }
        }
    );
    gen.into()

}
