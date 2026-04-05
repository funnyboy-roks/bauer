use proc_macro::TokenStream;

/// Place all tokens within the parenthesis before the item
#[proc_macro_attribute]
pub fn pre(mut attr: TokenStream, item: TokenStream) -> TokenStream {
    attr.extend(item);
    attr
}
