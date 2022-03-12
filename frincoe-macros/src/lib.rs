/*!
Macro definitions for frincoe-rpc.

See [the document of frincoe-rpc](../frincoe_rpc/index.html) for detailed document.
*/



#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_span)]
#![feature(proc_macro_expand)]
#![feature(doc_cfg)]
#![feature(extend_one)]



use proc_macro::TokenStream;
mod helpers;



mod make_client;
use make_client::make_client_impl;

/**
Make the target struct a client of given trait from specified adapter.

The path of the source trait will be relative to the source file where the macro is invoked.

Grammar:
```ignore
make_client!(impl "path/to/declaration/file"::Trait::Path
    [as Actual::Trait::Path] for TargetClient in adapter[(args)]);
```

The adapter must be a macro (usually proc macro),
for each trait item (constant, type, or function), it will be invoked once,
with the trait item as argument, surrounded with quotes.
If extra arguments are present, the adapter will be invoked with this format: `adapter(args; item)`.
Note that all the items comes with a semicolon at the end.

The `as` part is for when the trait was of other path than the path after the filename.
You may omit it, and the path after `impl` will be used.
Unfortunately, the full path including the file is still needed,
since our implement needs to see the trait definition to generate code.

Currently the trait can only be a bare trait without any qualifications,
this may be solved in later versions.
*/
#[proc_macro]
pub fn make_client(args: TokenStream) -> TokenStream {
    make_client_impl(args.into(), helpers::read_trait).into()
}



mod dispatch_cable;
use dispatch_cable::dispatch_cable_impl;

/**
Adapter for [`make_client!`] to make passive [`Cable`]s.

Apart from that `Self` should impl [`Cable`],
the return type `T`s of the methods should be `Extend<T> + Default`
to allow the macro to pack them as the final result.

Other declarations besides methods in the trait are ignored,
if it's needed, use a specialization (i.e. `default const V: T = ...;` etc.) to provide them a value.

See document of [frincoe-rpc](../frincoe_rpc/index.html) for the difference of passive and active clients,
and document of [`Cable`] for on which the adapter is used.

[`Cable`]: ../frincoe/cable/trait.Cable.html
 */
#[cfg(feature = "adapters")]
#[doc(cfg(any(feature = "adapters", feature = "full")))]
#[proc_macro]
pub fn dispatch_cable(args: TokenStream) -> TokenStream {
    dispatch_cable_impl(args.into()).into()
}
