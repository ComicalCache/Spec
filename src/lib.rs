#![feature(proc_macro_span)]

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::Not,
};

use quote::{format_ident, ToTokens};
use syn::{
    parse_macro_input, FnArg, Ident, ImplItem, ImplItemFn, Item, ItemFn, Meta, MetaList, Pat,
};

use proc_macro::{Span, TokenStream};

struct Pattern {
    pattern: String,
    func: String,
}

impl Pattern {
    pub fn new(token: &proc_macro2::TokenStream, func: String) -> Self {
        Pattern {
            pattern: format!("({token})"),
            func,
        }
    }
}

fn hash_ident(ident: &Ident, hasher: impl Hasher) -> String {
    format_ident!("{}_{:x}", ident, hasher.finish()).to_string()
}

fn hash_impl_ident(ident: &Ident, hasher: impl Hasher) -> String {
    format!("self.{}", hash_ident(ident, hasher))
}

fn has_attr(list: &MetaList) -> bool {
    list.path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .any(|seg| seg == "case")
}

#[proc_macro_attribute]
/// Creates a case for a specialized function.
///
/// The function must have an identical signature (except for parameter destructuring)
/// as the spec function.
///
/// The order of definition is the order of checking the cases, the first matching
/// pattern will be used.
pub fn case(attr: TokenStream, item: TokenStream) -> TokenStream {
    // randomize function name by using hashed pattern
    // => makes it so only `spec`` function is left with original name
    // => no multiple function declaration and normal function call syntax

    let mut hasher = DefaultHasher::new();
    attr.to_string().hash(&mut hasher);

    let mut input = parse_macro_input!(item as syn::ItemFn);
    // yes.. this double format_ident! is stupid, but oh well
    input.sig.ident = format_ident!("{}", hash_ident(&input.sig.ident, hasher));
    input.to_token_stream().into()
}

#[proc_macro_attribute]
/// Specify a function definition to be completed by an exhaustive
/// series of case implementations for the parameter cases.
pub fn spec(_: TokenStream, item: TokenStream) -> TokenStream {
    // build a match statement based on all the patterns
    // declared by #[case(...)] to pass to appropriate
    // function

    let mut input = parse_macro_input!(item as syn::ItemFn);

    // parse parameter names of function arguments to pass to cases later
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
    let params = params.join(",");

    // open the file of definition to access to all to this #[spec]
    // belonging #[case(...)] functions
    // !! THIS LIMITS THE CASES TO BE IN THE SAME FILE !!
    // TODO: add that spec can have a list of files to also check
    let span = Span::call_site();
    let source = span.source_file();
    let src_file =
        std::fs::read_to_string(source.path()).expect("Failed to open and read the source file");
    let src_file = syn::parse_file(&src_file).expect("Failed to parse source file");

    let patterns = iter_items(&input.sig.ident, &src_file.items);

    // build match statement with all the patterns and replace function body
    let mut match_arms = String::new();
    for Pattern { pattern, func } in patterns {
        match_arms.push_str(format!("{pattern}=>{func}({params}),").as_ref());
    }
    let block = format!("{{match({params}){{{match_arms}}}}}")
        .parse::<TokenStream>()
        .expect("Failed to parse function match arms");

    let block = Box::new(parse_macro_input!(block as syn::Block));
    input.block = block;

    input.to_token_stream().into()
}

/// Iter (recursively) over all items in a module, finding all #[case(...)] attributed
/// functions with same function name
fn iter_items(ident: &Ident, items: &Vec<Item>) -> Vec<Pattern> {
    // TODO: check if #[spec] function is Fn, ImplItemFn or in same mod
    let mut patterns: Vec<Pattern> = Vec::new();

    for item in items {
        match item {
            Item::Fn(func) => {
                if let Some(mut new) = item_fn(ident, &func) {
                    patterns.append(&mut new);
                }
            }
            Item::Impl(r#impl) => {
                for item in r#impl.items.iter() {
                    match item {
                        ImplItem::Fn(func) => {
                            if let Some(mut new) = impl_item_fn(ident, &func) {
                                patterns.append(&mut new);
                            }
                        }
                        _ => continue,
                    }
                }
            }
            Item::Mod(r#mod) => {
                if let Some(content) = &r#mod.content {
                    let mut new = iter_items(ident, &content.1);
                    patterns.append(&mut new);
                }
            }
            _ => continue,
        }
    }

    patterns
}

/// Find case functions on global scope
fn item_fn(ident: &Ident, func: &ItemFn) -> Option<Vec<Pattern>> {
    // ignore other functions (that may also have #[case(...)] attribute)
    if func.sig.ident != *ident {
        return None;
    }

    let mut patterns: Vec<Pattern> = Vec::new();
    for attr in func.attrs.iter() {
        match &attr.meta {
            Meta::List(attr) => {
                // check if fn has #[case(...)] attribute
                if has_attr(attr).not() {
                    continue;
                }

                let mut hasher = DefaultHasher::new();
                attr.tokens.to_string().hash(&mut hasher);
                patterns.push(Pattern::new(&attr.tokens, hash_ident(ident, hasher)));
            }
            _ => continue,
        }
    }

    Some(patterns)
}

/// Find case functions on impl scope
fn impl_item_fn(ident: &Ident, func: &ImplItemFn) -> Option<Vec<Pattern>> {
    // ignore other functions (that may also have #[case(...)] attribute)
    if func.sig.ident != *ident {
        return None;
    }

    let mut patterns: Vec<Pattern> = Vec::new();
    for attr in func.attrs.iter() {
        match &attr.meta {
            Meta::List(attr) => {
                // check if fn has #[case(...)] attribute
                if has_attr(attr).not() {
                    continue;
                }

                let mut hasher = DefaultHasher::new();
                attr.tokens.to_string().hash(&mut hasher);
                patterns.push(Pattern::new(&attr.tokens, hash_impl_ident(ident, hasher)));
            }
            _ => continue,
        }
    }

    Some(patterns)
}
