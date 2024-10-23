use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote_spanned, ToTokens};
use syn::{
    parse_macro_input, parse_quote, parse_quote_spanned, punctuated::Punctuated, spanned::Spanned,
    token::Comma, Arm, Attribute, Block, Expr, ExprMatch, ExprPath, FnArg, ItemFn, Meta, Pat, Path,
    PathSegment, Signature,
};

/// Create spanned compile error
fn error(span: &Span, e: &'static str) -> TokenStream {
    quote_spanned! {
        *span =>
        compile_error!(#e)
    }
    .into()
}

/// Parse the parameter names of the `#[spec]` function and make them into a comma
/// separated list
fn param_to_tuple(sig: &Signature) -> Punctuated<Expr, Comma> {
    let mut punc_seq: Punctuated<Expr, Comma> = Punctuated::new();

    for arg in sig.inputs.iter() {
        match arg {
            FnArg::Typed(arg) => match arg.pat.as_ref() {
                Pat::Ident(id) => {
                    punc_seq.push(Expr::from(ExprPath {
                        attrs: Vec::new(),
                        qself: None,
                        path: Path::from(PathSegment::from(id.ident.clone())),
                    }));
                }
                _ => panic!("Does not support non-ident arguments"),
            },
            _ => continue,
        }
    }

    punc_seq
}

#[proc_macro]
pub fn spec(item: TokenStream) -> TokenStream {
    // TODO: support impl with good auto complete

    // Brace input so it can be parsed as block
    let item = TokenStream2::from(item);
    let input: Block = parse_quote_spanned!( item.span() => { #item } );
    let mut stmts = input.stmts.iter();

    // Parse function specification
    let mut spec_func = if let Some(func) = stmts.next() {
        let stmt: TokenStream = func.to_token_stream().into();
        let mut func = parse_macro_input!(stmt as ItemFn);

        // TODO: check for flags like "strict name" or "strict parameters":
        // - strict name: enforces name of #[spec] and #[case] functions to be identical
        // - strict parameters: enforces parameters of #[spec] and #[case] functions to be identical
        let expected_attr: Attribute = parse_quote!( #[spec] );
        if let Some(attr_index) = func.attrs.iter().position(|attr| *attr == expected_attr) {
            // Attribute was only needed for identification
            func.attrs.remove(attr_index);
        } else {
            return error(
                &func.span(),
                "Expected the first function to be the function \
                pecification with the #[spec] attribute",
            );
        }
        func
    } else {
        return error(&input.span(), "spec!{{}} block must not be empty");
    };

    // Create the match statement
    let spec_func_parameters = param_to_tuple(&spec_func.sig);
    let mut expr_match: ExprMatch = parse_quote_spanned!( spec_func.span() =>
        #[allow(unused_parens)]
        match (#spec_func_parameters) {}
    );

    // Parse the pattern and function body of each case and assemble the match statement
    for stmt in stmts {
        let stmt: TokenStream = stmt.to_token_stream().into();
        let func = parse_macro_input!(stmt as ItemFn);

        // TODO: check for flags like "if":
        // - if: adds an if clause to the match arm
        if func.attrs.len() != 1 {
            return error(&func.span(), "Expected only #[case(...)]");
        }

        if let Meta::List(attr) = &func.attrs[0].meta {
            let pattern = &attr.tokens;
            let body = func.block;
            let arm: Arm = parse_quote_spanned!( spec_func.span() => (#pattern) => #body );
            expr_match.arms.push(arm);
        } else {
            return error(&func.span(), "Expected only #[case(...)]");
        }
    }

    let body: Block = parse_quote_spanned!( spec_func.span() => { #expr_match } );
    spec_func.block = Box::new(body);

    spec_func.to_token_stream().into()
}
