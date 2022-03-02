use frincoe_macros::make_client;

include!("interfaces.rs");

macro_rules! defaulting {
    ( fn $func:ident $args:tt -> $ret:ty; ) => {
        fn $func $args -> $ret {
            Default::default()
        }
    };
}

struct Target {
}

make_client!(impl "interfaces.rs"::pathing::MoreFns for Target in defaulting);

fn main() {
    use pathing::MoreFns;
    let t = Target {};
    t.f1(1);
    t.f2(123);
    t.f3(1.0, 1.0);
}
