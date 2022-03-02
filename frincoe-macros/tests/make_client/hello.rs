use frincoe_macros::make_client;

include!("interfaces.rs");

macro_rules! Error {
    ( fn $func:ident $args:tt -> $ret:ty; ) => {
        compile_error!(concat!(
            stringify!($func),
            stringify!($args),
            " -> ",
            stringify!($ret)
        ));
    };
}

struct Target {}

make_client!(impl "interfaces.rs"::SayHello for Target in Error);

fn main() {}
