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



mod inject_implement;
use inject_implement::inject_implement_impl;

/**
Inject the implement of a trait to a type using specified adapter.

The path of the source trait will be relative to the source file where the macro is invoked.

Grammar:
```text
inject_implement!(impl [{ trait Definition {} } | "path/to/definition/file"::Trait::Path]
    [as Actual::Trait::Path] for TargetClient in adapter[(args)]);
```

The adapter must be a macro (usually proc macro),
for each trait item (constant, type, or function), it will be invoked once,
with the trait item as argument, surrounded with quotes.
If extra arguments are present, the adapter will be invoked with this format: `adapter(args; item)`.
Note that all the items comes with a semicolon at the end.

The `as` part is for when the trait was of other path than the path after the filename.
If omitted, it will be inferred to be the same as the its path in the definition file,
or the name in the definition.
Unfortunately, the definition of the trait is needed however,
or there's no way to know the items of the trait.

Currently the trait can only be a bare trait without any qualifications,
this may be solved in later versions.
*/
#[proc_macro]
pub fn inject_implement(args: TokenStream) -> TokenStream {
    inject_implement_impl(args.into()).into()
}



mod dispatch_cable;
use dispatch_cable::dispatch_cable_impl;

/**
Adapter for [`inject_implement!`] to make passive [`Cable`]s.

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
#[doc(cfg(feature = "adapters"))]
#[proc_macro]
pub fn dispatch_cable(args: TokenStream) -> TokenStream {
    dispatch_cable_impl(args.into()).into()
}



mod forward_sub;
use forward_sub::forward_sub_impl;

/**
Adapter for [`inject_implement!`] that forwards the corresponding members of a specified member.

An extra argument specifying which member is used should be specified;
in some occasions when implementing static matters, like static methods, types and constants,
the type of the member should be given as well with the form `member: Type`.

Defaultly, only methods are forwarded, and other items of the trait can be provided by specializing.
Append `type` and/or `const` to the argument list to have types and/or constants forwarded.
 */
#[cfg(feature = "adapters")]
#[doc(cfg(feature = "adapters"))]
#[proc_macro]
pub fn forward_sub(args: TokenStream) -> TokenStream {
    forward_sub_impl(args.into()).into()
}
