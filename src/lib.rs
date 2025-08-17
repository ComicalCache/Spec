mod case_block;
use std::fmt::Display;

use case_block::CaseBlock;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::{
    parse_macro_input, parse_quote_spanned, punctuated::Punctuated, spanned::Spanned, token::Comma,
    Arm, Block, Expr, ExprMatch, ExprPath, ExprTuple, FnArg, ItemFn, Pat, PatIdent, PatTuple, Path,
    Signature,
};

/// Create spanned compile error.
fn error<T: Display>(span: &Span, e: T) -> TokenStream {
    syn::Error::new(*span, e).to_compile_error().into()
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
        // Recursively parse tuples to tuple expressions.
        Pat::Tuple(PatTuple { elems, .. }) => Some(Expr::from(ExprTuple {
            attrs: Vec::new(),
            paren_token: Default::default(),
            elems: elems.iter().filter_map(parse_arg).collect(),
        })),
        _ => unreachable!("Invalid function parameter..."),
    }
}

/// Parse the argument names of the specialized function and make them into a comma separated list.
fn param_to_tuple(sig: &Signature) -> Punctuated<Expr, Comma> {
    sig.inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(arg) => parse_arg(&*arg.pat),
            _ => None,
        })
        .collect()
}

#[proc_macro]
pub fn spec(item: TokenStream) -> TokenStream {
    // Brace input so it can be parsed as block.
    let item = TokenStream2::from(item);
    let input: Block = parse_quote_spanned!( item.span() => { #item } );
    let mut stmts = input.stmts.iter();

    // Parse function specification.
    let mut spec_func = if let Some(func) = stmts.next() {
        let func: TokenStream = func.into_token_stream().into();
        parse_macro_input!(func as ItemFn)
    } else {
        return error(&input.span(), "spec!{{}} block must not be empty");
    };

    // Parse parameters to expression for the match statement.
    let spec_func_parameters = param_to_tuple(&spec_func.sig);

    // Create the match statement.
    let mut expr_match: ExprMatch = parse_quote_spanned!( spec_func.span() =>
        #[allow(unused_parens)]
        match (#spec_func_parameters) {}
    );

    // Parse the pattern and function body of each case and assemble the match statement.
    for stmt in stmts {
        let stmt: TokenStream = stmt.to_token_stream().into();
        let CaseBlock { case, when, block } = parse_macro_input!(stmt as CaseBlock);

        let arm: Arm = match when {
            Some(when) => parse_quote_spanned!( spec_func.span() => (#case) if #when => #block ),
            None => parse_quote_spanned!( spec_func.span() => (#case) => #block ),
        };

        expr_match.arms.push(arm);
    }

    // Set the specialized function body to the match statement.
    let body: Block = parse_quote_spanned!( spec_func.span() => { #expr_match } );
    spec_func.block = Box::new(body);

    spec_func.into_token_stream().into()
}
