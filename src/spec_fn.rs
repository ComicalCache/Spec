use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Result, Signature, Token, Visibility,
};

pub struct SpecFn {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub sig: Signature,
    pub span: Span,
}

impl Parse for SpecFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let start = input.span();

        // Parse attributes first.
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;

        // Parse the rest of the function signature
        let vis: Visibility = input.parse()?;
        let sig: Signature = input.parse()?;
        let _: Token![;] = input.parse()?;

        Ok(SpecFn {
            attrs,
            vis,
            sig,
            span: start.join(input.span()).unwrap_or(start),
        })
    }
}
