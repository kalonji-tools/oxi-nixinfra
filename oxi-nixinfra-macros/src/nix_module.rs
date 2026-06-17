use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    braced, Attribute, Expr, FieldValue, Ident, Result, Token, Type,
};

/// Top-level input: `nix_module! { #[doc = "..."] Service { ... } }`
pub struct NixModuleDef {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub fields: Vec<FieldDef>,
    pub constructor: ConstructorDef,
    pub methods: Vec<MethodDef>,
}

pub struct FieldDef {
    pub name: Ident,
    pub ty: Type,
}

pub struct ConstructorDef {
    pub params: Vec<ConstructorParam>,
    pub body: Vec<FieldValue>,
}

pub struct ConstructorParam {
    pub name: Ident,
    pub ty: Type,
    #[allow(dead_code)]
    pub default: Option<Expr>,
}

pub struct MethodDef {
    pub name: Ident,
    pub params: Vec<MethodParam>,
    pub return_type: Type,
    pub impl_fn: Ident,
}

pub struct MethodParam {
    pub name: Ident,
    pub ty: Type,
}

impl MethodParam {
    /// Whether the parameter type is a reference (starts with `&`).
    pub fn is_ref(&self) -> bool {
        matches!(self.ty, Type::Reference(_))
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

impl Parse for NixModuleDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);

        // Parse `fields { ... }`
        let _: Ident = content.parse()?; // "fields"
        let fields = parse_fields(&content)?;
        content.parse::<Token![,]>().ok();

        // Parse `new(...) { ... }`
        let _: Ident = content.parse()?; // "new"
        let constructor = parse_constructor(&content)?;
        content.parse::<Token![,]>().ok();

        // Parse `methods { ... }`
        let _: Ident = content.parse()?; // "methods"
        let methods = parse_methods(&content)?;
        content.parse::<Token![,]>().ok();

        Ok(NixModuleDef {
            attrs,
            name,
            fields,
            constructor,
            methods,
        })
    }
}

fn parse_fields(input: ParseStream) -> Result<Vec<FieldDef>> {
    let content;
    braced!(content in input);
    let mut fields = Vec::new();
    while !content.is_empty() {
        let name: Ident = content.parse()?;
        content.parse::<Token![:]>()?;
        let ty: Type = content.parse()?;
        content.parse::<Token![,]>().ok();
        fields.push(FieldDef { name, ty });
    }
    Ok(fields)
}

fn parse_constructor(input: ParseStream) -> Result<ConstructorDef> {
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
        params.push(ConstructorParam { name, ty, default });
    }

    let body_content;
    braced!(body_content in input);
    let body: Punctuated<FieldValue, Token![,]> =
        body_content.parse_terminated(FieldValue::parse, Token![,])?;

    Ok(ConstructorDef {
        params,
        body: body.into_iter().collect(),
    })
}

fn parse_methods(input: ParseStream) -> Result<Vec<MethodDef>> {
    let content;
    braced!(content in input);
    let mut methods = Vec::new();
    while !content.is_empty() {
        content.parse::<Token![fn]>()?;
        let name: Ident = content.parse()?;

        // Parse params
        let params_content;
        syn::parenthesized!(params_content in content);
        let mut params = Vec::new();
        while !params_content.is_empty() {
            let param_name: Ident = params_content.parse()?;
            params_content.parse::<Token![:]>()?;
            let param_ty: Type = params_content.parse()?;
            if !params_content.is_empty() {
                params_content.parse::<Token![,]>()?;
            }
            params.push(MethodParam {
                name: param_name,
                ty: param_ty,
            });
        }

        // Parse -> ReturnType
        content.parse::<Token![->]>()?;
        let return_type: Type = content.parse()?;

        // Parse => impl_fn
        content.parse::<Token![=>]>()?;
        let impl_fn: Ident = content.parse()?;

        content.parse::<Token![;]>()?;

        methods.push(MethodDef {
            name,
            params,
            return_type,
            impl_fn,
        });
    }
    Ok(methods)
}

// ---------------------------------------------------------------------------
// Code generation
// ---------------------------------------------------------------------------

impl NixModuleDef {
    pub fn generate(&self) -> TokenStream {
        let sync_struct = self.gen_struct(false);
        let async_struct = self.gen_struct(true);
        let constructors = self.gen_constructors();
        let sync_methods = self.gen_pymethods(false);
        let async_methods = self.gen_pymethods(true);

        quote! {
            #sync_struct
            #async_struct
            #constructors
            #sync_methods
            #async_methods
        }
    }

    fn sync_name(&self) -> &Ident {
        &self.name
    }

    fn async_name(&self) -> Ident {
        format_ident!("Async{}", self.name)
    }

    fn gen_struct(&self, is_async: bool) -> TokenStream {
        let name = if is_async {
            self.async_name()
        } else {
            self.name.clone()
        };
        let attrs = &self.attrs;
        let field_defs: Vec<_> = self
            .fields
            .iter()
            .map(|f| {
                let fname = &f.name;
                let fty = &f.ty;
                quote! { pub(crate) #fname: #fty }
            })
            .collect();

        quote! {
            #(#attrs)*
            #[pyo3::prelude::pyclass(frozen)]
            pub struct #name {
                pub(crate) inner: std::sync::Arc<crate::host::HostInner>,
                #(#field_defs,)*
            }
        }
    }

    fn gen_constructors(&self) -> TokenStream {
        let sync_name = self.sync_name();
        let async_name = self.async_name();
        let ctor = self.gen_single_constructor(sync_name);
        let async_ctor = self.gen_single_constructor(&async_name);
        quote! {
            #ctor
            #async_ctor
        }
    }

    fn gen_single_constructor(&self, struct_name: &Ident) -> TokenStream {
        let params: Vec<_> = self
            .constructor
            .params
            .iter()
            .map(|p| {
                let name = &p.name;
                let ty = &p.ty;
                quote! { #name: #ty }
            })
            .collect();
        let field_inits: Vec<_> = self
            .constructor
            .body
            .iter()
            .map(|fv| quote! { #fv })
            .collect();

        quote! {
            impl #struct_name {
                pub fn new(inner: std::sync::Arc<crate::host::HostInner>, #(#params),*) -> Self {
                    Self {
                        inner,
                        #(#field_inits,)*
                    }
                }
            }
        }
    }

    fn gen_pymethods(&self, is_async: bool) -> TokenStream {
        let struct_name = if is_async {
            self.async_name()
        } else {
            self.name.clone()
        };

        let methods: Vec<_> = self
            .methods
            .iter()
            .map(|m| {
                if is_async {
                    self.gen_async_method(m)
                } else {
                    self.gen_sync_method(m)
                }
            })
            .collect();

        let repr = self.gen_repr(is_async);

        // Don't generate empty pymethods block when there are no methods and no repr
        if methods.is_empty() && repr.is_empty() {
            return quote! {};
        }

        quote! {
            #[pyo3::prelude::pymethods]
            impl #struct_name {
                #(#methods)*
                #repr
            }
        }
    }

    fn gen_sync_method(&self, method: &MethodDef) -> TokenStream {
        let name = &method.name;
        let ret_ty = &method.return_type;
        let impl_fn = &method.impl_fn;

        let params: Vec<_> = method
            .params
            .iter()
            .map(|p| {
                let pname = &p.name;
                let pty = &p.ty;
                quote! { #pname: #pty }
            })
            .collect();

        // Build impl fn call args: &self.inner, &self.field1, ..., extra_arg1, ...
        let mut call_args = vec![quote! { &self.inner }];
        for f in &self.fields {
            let fname = &f.name;
            call_args.push(quote! { &self.#fname });
        }
        for p in &method.params {
            let pname = &p.name;
            call_args.push(quote! { #pname });
        }

        quote! {
            fn #name(&self, #(#params),*) -> pyo3::prelude::PyResult<#ret_ty> {
                crate::helpers::wrap_sync(&self.inner, #impl_fn(#(#call_args),*))
            }
        }
    }

    fn gen_async_method(&self, method: &MethodDef) -> TokenStream {
        let name = &method.name;
        let impl_fn = &method.impl_fn;

        let params: Vec<_> = method
            .params
            .iter()
            .map(|p| {
                let pname = &p.name;
                let pty = &p.ty;
                quote! { #pname: #pty }
            })
            .collect();

        // Clone statements for fields
        let mut clones = vec![quote! { let inner = self.inner.clone(); }];
        for f in &self.fields {
            let fname = &f.name;
            clones.push(quote! { let #fname = self.#fname.clone(); });
        }

        // Clone/to_owned statements for ref method params
        for p in &method.params {
            let pname = &p.name;
            if p.is_ref() {
                clones.push(quote! { let #pname = #pname.to_owned(); });
            }
        }

        // Build impl fn call args inside async block
        let mut call_args = vec![quote! { &inner }];
        for f in &self.fields {
            let fname = &f.name;
            call_args.push(quote! { &#fname });
        }
        for p in &method.params {
            let pname = &p.name;
            if p.is_ref() {
                call_args.push(quote! { &#pname });
            } else {
                call_args.push(quote! { #pname });
            }
        }

        quote! {
            fn #name<'py>(&self, py: pyo3::prelude::Python<'py>, #(#params),*) -> pyo3::prelude::PyResult<pyo3::prelude::Bound<'py, pyo3::prelude::PyAny>> {
                #(#clones)*
                pyo3_async_runtimes::tokio::future_into_py(py, async move {
                    #impl_fn(#(#call_args),*)
                        .await
                        .map_err(crate::helpers::backend_err_to_py)
                })
            }
        }
    }

    fn gen_repr(&self, is_async: bool) -> TokenStream {
        let prefix = if is_async {
            format!("Async{}", self.name)
        } else {
            self.name.to_string()
        };

        if self.fields.is_empty() {
            let lit = format!("<{prefix}>");
            quote! {
                fn __repr__(&self) -> String {
                    #lit.to_owned()
                }
            }
        } else if self.fields.len() == 1 {
            let field = &self.fields[0];
            let fname = &field.name;
            // Check if the field type is Option<...>
            if is_option_type(&field.ty) {
                // Don't generate __repr__ for Option fields — let the module handle it manually
                quote! {}
            } else {
                let fmt = format!("<{prefix} {{}}>");
                quote! {
                    fn __repr__(&self) -> String {
                        format!(#fmt, self.#fname)
                    }
                }
            }
        } else {
            // Multiple fields — don't generate, let module handle it
            quote! {}
        }
    }
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        type_path
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "Option")
    } else {
        false
    }
}
