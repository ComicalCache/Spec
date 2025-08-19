use syn::{
    parse::{Parse, ParseStream},
    parse_quote_spanned,
    punctuated::Punctuated,
    token::{Comma, Paren},
    Arm, Block, Expr, ExprMatch, ExprPath, ExprTuple, FnArg, ItemFn, Pat, PatIdent, PatStruct,
    PatTuple, PatTupleStruct, Path, Result,
};

use crate::SpecFn;
pub struct Spec {
    pub r#fn: ItemFn,
}

/// Parses an argument to an Expresion.
fn parse_arg(pat: &Pat) -> Option<Expr> {
    match pat {
        // Parse ident as expr.
        Pat::Ident(PatIdent { ident, .. }) => Some(Expr::from(ExprPath {
            attrs: Vec::new(),
            qself: None,
            path: Path::from(ident.clone()),
        })),
        // Skip wildcards.
        Pat::Wild(_) => None,
        // Recursively parse tuples and tuple structs to tuple expressions.
        Pat::Tuple(PatTuple { elems, .. }) | Pat::TupleStruct(PatTupleStruct { elems, .. }) => {
            Some(Expr::from(ExprTuple {
                attrs: Vec::new(),
                paren_token: Paren::default(),
                elems: elems.iter().filter_map(parse_arg).collect(),
            }))
        }
        Pat::Struct(PatStruct { fields, .. }) => Some(Expr::from(ExprTuple {
            attrs: Vec::new(),
            paren_token: Paren::default(),
            elems: fields
                .iter()
                .map(|field| &*field.pat)
                .filter_map(parse_arg)
                .collect(),
        })),
        _ => unreachable!("Invalid function parameter..."),
    }
}

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        let SpecFn {
            attrs,
            vis,
            sig,
            span,
        } = input.parse()?;

        // Parse parameters to expression for the match statement.
        let spec_func_parameters: Punctuated<Expr, Comma> = sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                FnArg::Typed(arg) => parse_arg(&arg.pat),
                FnArg::Receiver(_) => None,
            })
            .collect();

        // Create the match statement.
        let mut expr_match: ExprMatch = parse_quote_spanned!( span =>
            #[allow(unused_parens)]
            match (#spec_func_parameters) {}
        );

        // Parse the pattern and function body of each case and assemble the match statement.
        while !input.is_empty() {
            let arm: Arm = input.parse()?;
            expr_match.arms.push(arm);
        }

        // Set the specialized function body to the match statement.
        let body: Block = parse_quote_spanned!( span => { #expr_match } );
        let block = Box::new(body);

        Ok(Spec {
            r#fn: ItemFn {
                attrs,
                vis,
                sig,
                block,
            },
        })
    }
}
