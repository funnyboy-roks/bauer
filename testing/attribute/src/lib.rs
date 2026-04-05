use proc_macro::TokenStream;

/// Place all tokens within the parenthesis before the item
#[proc_macro_attribute]
pub fn pre(mut attr: TokenStream, item: TokenStream) -> TokenStream {
    attr.extend(item);
    attr
}

/// Blank attribute that provides #[my_attribute] for documentation
#[proc_macro_attribute]
pub fn my_attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Blank attribute that provides #[my_attribute2] for documentation
#[proc_macro_attribute]
pub fn my_attribute2(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
