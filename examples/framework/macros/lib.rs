extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse_macro_input!(item as syn::ItemFn);

    let vis = &item.vis;
    let attrs = &item.attrs;
    let name = &item.sig.ident;
    let test_name = name.to_string();
    let body = &*item.block;
    let expanded = quote! {
        #[test_case]
        #(#attrs)*
        #vis fn #name() -> mimicaw::Test<mimicaw_framework::TestCase> {
            mimicaw::Test::test(
                #test_name,
                Box::pin(async {
                    #body
                })
            )
        }
    };

    TokenStream::from(expanded)
}
