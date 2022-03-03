/*!
Macro definitions for frincoe-rpc.

See [the document of frincoe-rpc](../frincoe_rpc/index.html) for detailed document.
*/

#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_span)]
#![feature(proc_macro_expand)]

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
    [as Actual::Trait::Path] for TargetClient in adapter);
```

The adapter must be a macro (usually proc macro),
for each trait item (constant, type, or function), it will be invoked once,
with the trait item as argument, surrounded with quotes.

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
