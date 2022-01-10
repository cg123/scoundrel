mod shader_preprocessor;

use crate::shader_preprocessor::ShaderPreprocessor;
use once_cell::sync::OnceCell;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{proc_macro_error, Diagnostic, Level};
use quote::ToTokens;
use std::sync::RwLock;
use syn::{parse_macro_input, LitStr};

static PREPROCESSOR: OnceCell<RwLock<ShaderPreprocessor>> = OnceCell::new();

#[proc_macro]
#[proc_macro_error]
pub fn wgsl_module(input: TokenStream) -> TokenStream {
    let rel_path = parse_macro_input!(input as LitStr).value();

    let lock = PREPROCESSOR.get_or_init(|| RwLock::new(ShaderPreprocessor::default()));
    let data = {
        let mut pp = lock.write().unwrap();
        match pp.get(rel_path, &vec![]) {
            Ok(data) => data.to_string(),
            Err(e) => {
                Diagnostic::new(Level::Error, format!("{}", e)).abort();
            }
        }
    };

    LitStr::new(&data, Span::call_site())
        .to_token_stream()
        .into()
}
