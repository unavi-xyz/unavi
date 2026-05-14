use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemFn, parse_macro_input};

/// `bevy_hsd`
///
/// Adds two test cases.
/// Calls a setup function before or after spawning a HSD.
///
/// Useful for testing both initialization and runtime update handling.
#[proc_macro_attribute]
pub fn pre_post_cases(attr: TokenStream, item: TokenStream) -> TokenStream {
    let setup_fn = parse_macro_input!(attr as Ident);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let vis = &input.vis;
    let block = &input.block;

    let expanded = quote! {
        #[tracing_test::traced_test]
        #[rstest]
        #[case(true)]
        #[case(false)]
        #vis fn #fn_name(mut ctx: TestContext, #[case] pre: bool) {
            if pre {
                #setup_fn(&ctx);
            }

            ctx.spawn_hsd();

            if !pre {
                #setup_fn(&ctx);
            }

            ctx.app.update();

            #block
        }
    };

    TokenStream::from(expanded)
}
