use quote::ToTokens;

mod spec;
use spec::Spec;

mod spec_fn;
use spec_fn::SpecFn;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn spec(item: TokenStream) -> TokenStream {
    parse_macro_input!(item as Spec)
        .r#fn
        .to_token_stream()
        .into()
}
