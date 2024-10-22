#![feature(proc_macro_span)]

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::Not,
};

use quote::{format_ident, ToTokens};
use syn::{parse_macro_input, FnArg, Item, Meta, Pat};

use proc_macro::{Span, TokenStream};

#[proc_macro_attribute]
pub fn case(attr: TokenStream, item: TokenStream) -> TokenStream {
    // randomize function name by using hashed pattern to match
    // => makes it so only base function is left with original name
    // => no multiple function declaration

    let mut hasher = DefaultHasher::new();
    attr.to_string().hash(&mut hasher);

    let mut input = parse_macro_input!(item as syn::ItemFn);
    input.sig.ident = format_ident!("{}{:x}", input.sig.ident, hasher.finish());
    input.to_token_stream().into()
}

#[proc_macro_attribute]
pub fn spec(_: TokenStream, item: TokenStream) -> TokenStream {
    // build a match statement based on all the patterns
    // declared by #[fn_match(...)] to pass to appropriate
    // function

    let mut input = parse_macro_input!(item as syn::ItemFn);

    // parse parameter names of function to pass to "function variants" later
    let mut params: Vec<String> = Vec::new();
    for arg in input.sig.inputs.iter() {
        match arg {
            FnArg::Typed(arg) => match arg.pat.as_ref() {
                Pat::Ident(id) => params.push(id.ident.to_string()),
                _ => panic!("Does not support non-ident arguments"),
            },
            _ => continue,
        }
    }
    let params = params.join(", ");

    // open original source file to get access to all function definitions
    // containing #[fn_match(...)] attributes
    // !! THIS LIMITS THE "FUNCTION VARIANTS" TO THE SAME SOURCE FILE !!
    let span = Span::call_site();
    let source = span.source_file();
    let src_file =
        std::fs::read_to_string(source.path()).expect("Failed to open and read the source file");
    let src_file = syn::parse_file(&src_file).expect("Failed to parse source file");

    // (pattern, function name)
    let mut patterns: Vec<(String, String)> = Vec::new();

    // find all #[fn_match(...)] attributed functions with same function name,
    // save the pattern and (hashed!) function name
    for item in src_file.items {
        match item {
            Item::Fn(func) => {
                // ignore other functions (that may also have #[fn_match(...)] attribute)
                if func.sig.ident != input.sig.ident {
                    continue;
                }

                for attr in func.attrs {
                    match attr.meta {
                        Meta::List(attr) => {
                            // check if fn has #[fn_match(...)] attribute
                            if attr
                                .path
                                .segments
                                .iter()
                                .map(|seg| seg.ident.to_string())
                                .any(|seg| seg == "case")
                                .not()
                            {
                                continue;
                            }

                            let mut hasher = DefaultHasher::new();
                            attr.tokens.to_string().hash(&mut hasher);
                            patterns.push((
                                format!("({})", attr.tokens.to_string()),
                                format!("{}{:x}", input.sig.ident.to_string(), hasher.finish()),
                            ));
                        }
                        _ => continue,
                    }
                }
            }
            _ => continue,
        }
    }

    // build match statement with all the patterns and replace function body
    let mut match_arms = String::new();
    for (pattern, func) in patterns {
        match_arms.push_str(format!("{pattern} => {func}({params}), ").as_ref());
    }
    let block = format!("{{ match ({params}) {{ {match_arms} }} }}")
        .parse::<TokenStream>()
        .expect("Failed to parse function match arms");

    let block = Box::new(parse_macro_input!(block as syn::Block));
    input.block = block;

    input.to_token_stream().into()
}
