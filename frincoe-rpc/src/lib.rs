/*!
Remote procedure call for [frincoe](../frincoe/index.html).
Designed to be capable to all kinds of procedure calls.

Procedures and messages themselves are defined by the implements.

A struct implements the methods in a interface is called a provider,
and a struct actually be used by users is called a client.
The two concepts are a description to a struct's usage,
since there's not a clear distinction between them.

For clients, procedure calls are direct: they act the same as normal methods.
But for a provider, it may be either passive or active to clients.
A passive provider is also the same as normal structs:
all procedure calls are automatically delivered to corresponding method implements.
And an active provider is like a server (see also [`Server`]),
which needs to manually fetch the incoming responses (see also [`FetchCall`])
and dispatch it to the correct method (see also [`Dispatcher`]).
Passive providers are for 'local' calls, like interthreadical calls,
and our implement to active calls also rely on the fact that all active providers
can also be passive providers.
And active providers are for remote calls, e.g. IPC, and what the term RPC usually refer to.

# Example

A basic example for a passive provider.

```
use frincoe_rpc::{inject_implement, Connection};
use frincoe_macros::forward_sub;
// Define an interface
trait SayHello {
    fn hello(&self, name: &str) -> String;
}

// Implement a provider for the interface
struct HelloProvider {
    word: &'static str,
}

impl SayHello for HelloProvider {
    fn hello(&self, name: &str) -> String {
        format!("{} {}", self.word, name)
    }
}

// Implement a client
struct DirectCall<T> {
    data: T,
}

// Not required by `inject_implement!`, and unnecessary here,
// but foundamental to a client.
impl<T> Connection for DirectCall<T> {
    type Error = ();
    fn disconnect(&self) -> Result<(), Self::Error> { Ok(()) }
}

inject_implement! {
    impl {
        trait SayHello {
            fn hello(&self, name: &str) -> String;
        }
    } for DirectCall<HelloProvider> in forward_sub(data)
}

// Usage
let handle = DirectCall { data: HelloProvider { word: "hello" } };
assert_eq!(handle.hello("world"), "hello world");
let handle = DirectCall { data: HelloProvider { word: "hi" } };
assert_eq!(handle.hello("blah"), "hi blah");
```
*/



use core::task::Poll;



/**
Something that can be disconnected, used for clients and connections accepted by [`Server`]s.

This is passive by default. For detailed document, see the [crate-level document](self).
*/
pub trait Connection {
    /// Possible errors of the connection.
    type Error;
    /// Disconnect from another end of the connection.
    ///
    /// The consequence of calling it for multiple times is implement-defined.
    /// Usually, this should only be called once.
    fn disconnect(&self) -> Result<(), Self::Error>;
}

/**
Serve on some address to accept incoming connections, used for active providers.
*/
pub trait Server: Sized {
    /// The address to listen on
    type Address;
    /// The type of an incoming client
    type Incoming: Connection;
    /// Possible errors during serving
    type Error;
    fn serve(addr: Self::Address) -> Result<Self, Self::Error>;
    fn accept(&self) -> Result<Self::Incoming, Self::Error>;
    fn shutdown(self) -> Result<(), Self::Error>;
}

/**
Fetch an incoming procedure call.

There's no mechanic to get the origin message,
to get them, plug an adapter to the provider to extract the origin request and response.
*/
pub trait FetchCall: Connection {
    /// Check if there is pending calls to be polled.
    fn has_call() -> bool;
    /// Poll a call and process it, returning whether there is a call.
    fn poll_call() -> Poll<()>;
}

/**
Dispatch request into implements, returning response;
used for [`make_dispatcher`](../frincoe_macros/macro.make_dispatcher.html).

It's usually generated by `make_dispatcher`,
to ensure response types are matched with the request types.
Manully implementing this is not encouraged.

For detailed document, see [crate-level document](self).
*/
pub trait Dispatcher {
    /// Incoming requests, including all the underlying functions' parameters.
    type Request;
    /// Responses returned by underlying functions.
    type Response;
    /// Dispatch the request to functions according to their types,
    /// returning respective response.
    fn dispatch(&mut self, request: Self::Request) -> Self::Response;
}



#[doc(inline)]
pub use frincoe_macros::inject_implement;
#[doc(inline)]
pub use frincoe_macros::make_dispatcher;



#[cfg(test)]
mod tests {}
