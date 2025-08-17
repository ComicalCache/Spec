use proc_macro2::TokenStream;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Attribute, Block, Meta, Result,
};

pub struct CaseBlock {
    pub case: TokenStream,
    pub when: Option<TokenStream>,
    pub block: Block,
}

impl Parse for CaseBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse the attribute.
        let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
        if attrs.is_empty() || attrs.len() > 2 {
            return Err(input
                .error("Expected only the `case` attribute and the optional `when` attribute."));
        }

        let mut case_attr = None;
        let mut when_attr = None;

        // Parse the inside of the attributes.
        for attr in attrs {
            match attr.meta {
                Meta::List(meta) if attr.path().is_ident("case") => case_attr = Some(meta.tokens),
                Meta::List(meta) if attr.path().is_ident("when") => when_attr = Some(meta.tokens),
                _ => {
                    return Err(input.error(
                        "The `case` and the optional `when` attribute must be list attributes.",
                    ))
                }
            }
        }

        // Require the case attribute.
        let case_attr = case_attr.ok_or_else(|| input.error("Missing `case` attribute."))?;

        // Parse the code block.
        let content;
        let brace_token = braced!(content in input);
        let stmts = content.call(syn::Block::parse_within)?;

        Ok(CaseBlock {
            case: case_attr,
            when: when_attr,
            block: Block { brace_token, stmts },
        })
    }
}
