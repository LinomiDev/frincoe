use frincoe_macros::make_client;
use pathing::MoreFns;

include!("interfaces.rs");

macro_rules! defaulting {
    ( fn $func:ident $args:tt -> $ret:ty; ) => {
        fn $func $args -> $ret {
            Default::default()
        }
    };
}

struct Target {}

make_client!(impl "interfaces.rs"::pathing::MoreFns as MoreFns for Target in defaulting);

fn main() {}
