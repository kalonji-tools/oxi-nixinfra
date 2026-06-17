use proc_macro::TokenStream;
use syn::parse_macro_input;

mod nix_module;
mod register;

/// Generates sync and async PyO3 wrapper structs + pymethods from a module definition.
#[proc_macro]
pub fn nix_module(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as nix_module::NixModuleDef);
    def.generate().into()
}

/// Generates Host/AsyncHost factory methods for registered modules.
#[proc_macro]
pub fn register_modules(input: TokenStream) -> TokenStream {
    let def = parse_macro_input!(input as register::RegisterModulesDef);
    def.generate().into()
}
