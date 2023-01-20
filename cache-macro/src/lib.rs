/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2021-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use proc_macro::TokenStream;

use darling::FromMeta;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, parse_quote, Expr, ExprPath, FnArg, Path, PathSegment};

const DEFAULT_CACHE_TIMEOUT: u32 = 3600;

#[derive(Debug, darling::FromMeta)]
struct CacheParameter {
    #[darling(default)]
    /// Cache timeout option (in second)
    timeout: Option<u32>,
}

fn parse_attribute(args: syn::AttributeArgs) -> CacheParameter {
    match CacheParameter::from_list(&args) {
        Ok(param) => param,
        Err(e) => {
            panic!("{}", e.to_string());
        }
    }
}

fn parse_fn(item: syn::Item) -> syn::ItemFn {
    if let syn::Item::Fn(func) = item {
        func
    } else {
        panic!("You should only attach cache attribute to a function.");
    }
}

///
/// https://stackoverflow.com/questions/71480280/how-do-i-pass-arguments-from-a-generated-function-to-another-function-in-a-proce
fn transform_args(args: &Punctuated<syn::FnArg, Comma>) -> Punctuated<syn::Expr, Comma> {
    let idents = args.iter().filter_map(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                return Some(pat_ident.ident);
            }
        }
        None
    });

    let mut punctuated: Punctuated<syn::Ident, Comma> = Punctuated::new();
    idents.for_each(|ident| punctuated.push(ident));

    parse_quote!((#punctuated))
}

#[proc_macro_attribute]
pub fn cache(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as syn::AttributeArgs);
    let item = parse_macro_input!(item as syn::Item);

    // println!("item = {:#?}", item);

    // Parse cache parameter
    let param = parse_attribute(args);
    let timeout = param.timeout.unwrap_or(DEFAULT_CACHE_TIMEOUT);

    // Parse function signature
    let syn::ItemFn { attrs, vis, sig, block } = parse_fn(item);

    // Parse function parameter
    let func_all_args = transform_args(&sig.inputs);

    // New name

    let result = proc_macro::TokenStream::from(quote! {
        #(#attrs)* #vis #sig {

            println!("before");

            let result = #block;

            println!("after");

            result
        }
    });

    println!("{:?}", result.to_string());
    result
}
