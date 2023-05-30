extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, AttributeArgs, DeriveInput, Meta, NestedMeta};
#[proc_macro_derive(PostApi)]
pub fn post_api(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let ident = ast.ident;
    let gen = quote!(
        impl PostApi for #ident{
             fn post(&self)->serde_json::Value{
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

    let name = name.replace("message", "msg");

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
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::ItemFn = syn::parse(item).unwrap();
    let attrs = parse_macro_input!(attr as AttributeArgs);

    let mut ser_name = String::new();
    let mut ser_desc: String= String::new();
    let mut ser_cmd: String = String::new();
    let mut ser_alias: String = String::new();
    let mut ser_exclude: bool = false;
    for a in attrs {
        if let NestedMeta::Meta(inner) = a {
            if let Meta::NameValue(nv) = inner {
                match nv.path.get_ident().unwrap().to_string().as_str() {
                    "name" => {
                        if let syn::Lit::Str(lit) = nv.lit {
                            ser_name = lit.value();
                        }
                    }
                    "desc" => {
                        if let syn::Lit::Str(lit) = nv.lit {
                            ser_desc = lit.value();
                        }
                    }
                    "cmd" => {
                        if let syn::Lit::Str(lit) = nv.lit {
                            ser_cmd = lit.value();
                        }
                    }
                    "alias" => {
                        if let syn::Lit::Str(lit) = nv.lit {
                            ser_alias = lit.value();
                        }
                    }
                    "exclude" => {
                        if let syn::Lit::Bool(lit) = nv.lit {
                            ser_exclude = lit.value;
                        }
                    }
                    _ => {}
                }

            }
        }
    }

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
        impl ::rustqq::app::app::EventHandler for #ident{
            async fn register(&self,event:&::rustqq::event::Event)->Result<(),Box<dyn std::error::Error>>{

                #block
            }
        }

        impl ::rustqq::app::service::IntoService for #ident{
            fn into_service(self)->::rustqq::app::service::Service{
                ::rustqq::app::service::Service{
                    info: ::rustqq::app::service::ServiceInfo{
                        name:#ser_name.to_string(),
                        description:#ser_desc.to_string(),
                        command:#ser_cmd.to_string(),
                        alias:#ser_alias.to_string(),
                        exclude:#ser_exclude,
                    },
                    handler:Box::new(self),
                }
            }
        }
    );
    gen.into()
}
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::ItemFn = syn::parse(item).unwrap();
    let attrs = parse_macro_input!(attr as AttributeArgs);

    let mut ser_name = String::new();
    let mut ser_desc: String= String::new();
    let mut ser_cmd: String = String::new();
    let mut ser_alias: String = String::new();
    let mut ser_exclude: bool = false;
    for a in attrs {
        if let NestedMeta::Meta(inner) = a {
            if let Meta::NameValue(nv) = inner {
                match nv.path.get_ident().unwrap().to_string().as_str() {
                    "name" => {
                        if let syn::Lit::Str(lit) = nv.lit {
                            ser_name = lit.value();
                        }
                    }
                    "desc" => {
                        if let syn::Lit::Str(lit) = nv.lit {
                            ser_desc = lit.value();
                        }
                    }
                    "cmd" => {
                        if let syn::Lit::Str(lit) = nv.lit {
                            ser_cmd = lit.value();
                        }
                    }
                    "alias" => {
                        if let syn::Lit::Str(lit) = nv.lit {
                            ser_alias = lit.value();
                        }
                    }
                    "exclude" => {
                        if let syn::Lit::Bool(lit) = nv.lit {
                            ser_exclude = lit.value;
                        }
                    }
                    _ => {}
                }

            }
        }
    }

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
        impl ::rustqq::app::app::Command for #ident{
            async fn proc(&self,msg_event: ::rustqq::event::MsgEvent)->Result<(),Box<dyn std::error::Error>>{
                #block

            }
        }

        impl ::rustqq::app::service::IntoService for #ident{
            fn into_service(self)->::rustqq::app::service::Service{
                ::rustqq::app::service::Service{
                    info: ::rustqq::app::service::ServiceInfo{
                        name:#ser_name.to_string(),
                        description:#ser_desc.to_string(),
                        command:#ser_cmd.to_string(),
                        alias:#ser_alias.to_string(),
                        exclude:#ser_exclude,
                    },
                    handler:Box::new(self),
                }
            }
        }

    );
    gen.into()
}