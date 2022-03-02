use frincoe_macros::make_client;

make_client!(impl "interfaces.rs"::NotExisted for Target in test);

fn main() {}
