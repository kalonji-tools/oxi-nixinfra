use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, Result, Token, Type};

pub struct RegisterModulesDef {
    pub host_ident: Ident,
    pub async_host_ident: Ident,
    pub entries: Vec<ModuleEntry>,
}

pub struct ModuleEntry {
    pub method_name: Ident,
    pub params: Vec<ModuleParam>,
    pub module_path: Ident,
    pub module_type: Ident,
}

pub struct ModuleParam {
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
}

impl Parse for RegisterModulesDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let host_ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let async_host_ident: Ident = input.parse()?;
        input.parse::<Token![;]>()?;

        let mut entries = Vec::new();
        while !input.is_empty() {
            let method_name: Ident = input.parse()?;

            let params_content;
            syn::parenthesized!(params_content in input);
            let mut params = Vec::new();
            while !params_content.is_empty() {
                let name: Ident = params_content.parse()?;
                params_content.parse::<Token![:]>()?;
                let ty: Type = params_content.parse()?;
                let default = if params_content.peek(Token![=]) {
                    params_content.parse::<Token![=]>()?;
                    Some(params_content.parse::<Expr>()?)
                } else {
                    None
                };
                if !params_content.is_empty() {
                    params_content.parse::<Token![,]>()?;
                }
                params.push(ModuleParam { name, ty, default });
            }

            input.parse::<Token![->]>()?;
            let module_path: Ident = input.parse()?;
            input.parse::<Token![::]>()?;
            let module_type: Ident = input.parse()?;
            input.parse::<Token![,]>().ok();

            entries.push(ModuleEntry {
                method_name,
                params,
                module_path,
                module_type,
            });
        }

        Ok(RegisterModulesDef {
            host_ident,
            async_host_ident,
            entries,
        })
    }
}

impl RegisterModulesDef {
    pub fn generate(&self) -> TokenStream {
        let host = &self.host_ident;
        let async_host = &self.async_host_ident;

        let sync_methods: Vec<_> = self.entries.iter().map(|e| e.gen_factory(false)).collect();
        let async_methods: Vec<_> = self.entries.iter().map(|e| e.gen_factory(true)).collect();

        quote! {
            #[pyo3::prelude::pymethods]
            impl #host {
                #(#sync_methods)*
            }

            #[pyo3::prelude::pymethods]
            impl #async_host {
                #(#async_methods)*
            }
        }
    }
}

impl ModuleEntry {
    fn gen_factory(&self, is_async: bool) -> TokenStream {
        let method_name = &self.method_name;
        let module_type = if is_async {
            format_ident!("Async{}", self.module_type)
        } else {
            self.module_type.clone()
        };

        let params: Vec<_> = self
            .params
            .iter()
            .map(|p| {
                let name = &p.name;
                let ty = &p.ty;
                quote! { #name: #ty }
            })
            .collect();

        let args: Vec<_> = self.params.iter().map(|p| &p.name).collect();

        let has_defaults = self.params.iter().any(|p| p.default.is_some());
        let sig_attr = if has_defaults {
            let sig_parts: Vec<_> = self
                .params
                .iter()
                .map(|p| {
                    let name = &p.name;
                    if let Some(default) = &p.default {
                        quote! { #name = #default }
                    } else {
                        quote! { #name }
                    }
                })
                .collect();
            quote! { #[pyo3(signature = (#(#sig_parts),*))] }
        } else {
            quote! {}
        };

        let module_path = &self.module_path;

        quote! {
            #sig_attr
            fn #method_name(&self, #(#params),*) -> crate::modules::#module_path::#module_type {
                crate::modules::#module_path::#module_type::new(self.inner.clone(), #(#args),*)
            }
        }
    }
}
