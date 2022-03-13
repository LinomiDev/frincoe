use frincoe::cable::{ArrayCable, Bundle, Cable};
use frincoe_macros::{dispatch_cable, inject_implement};
use frincoe_rpc::Connection;



// The interface
/// Say hello and bye :)
trait Greet {
    fn hello(&mut self, name: &str) -> Bundle<String> {
        Bundle::from_single(format!("hello {}", name))
    }
    fn bye(&mut self, name: &str) -> Bundle<String> {
        Bundle::from_single(format!("bye {}", name))
    }
}



// The providers to the interface
/// Greeting with a different hello
struct VaryGreet {
    hello: String,
}

impl VaryGreet {
    pub fn new(hello: impl ToString) -> Self {
        Self {
            hello: hello.to_string(),
        }
    }
}

impl Greet for VaryGreet {
    fn hello(&mut self, name: &str) -> Bundle<String> {
        Bundle::from_single(format!("{} {}", self.hello, name))
    }
}

/// Add a count before the greeting
struct CountedGreet {
    count: usize,
}

impl CountedGreet {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Greet for CountedGreet {
    fn hello(&mut self, name: &str) -> Bundle<String> {
        self.count += 1;
        Bundle::from_single(format!("hello #{} {}", self.count - 1, name))
    }
    fn bye(&mut self, name: &str) -> Bundle<String> {
        self.count += 1;
        Bundle::from_single(format!("bye #{} {}", self.count - 1, name))
    }
}



// Implement the cables
inject_implement! {
    impl {
        trait Greet {
            fn hello(&mut self, name: &str) -> Bundle<String> {
                format!("hello {}", name)
            }
            fn bye(&mut self, name: &str) -> Bundle<String> {
                format!("bye {}", name)
            }
        }
    } as Greet for ArrayCable<VaryGreet> in dispatch_cable
}

inject_implement! {
    impl {
        trait Greet {
            fn hello(&mut self, name: &str) -> Bundle<String>;
            fn bye(&mut self, name: &str) -> Bundle<String>;
        }
    } as Greet for ArrayCable<&mut dyn Greet> in dispatch_cable
}



fn main() {
    // Static arrays
    let mut cable = ArrayCable::<VaryGreet>::new();
    cable.add_connection(VaryGreet::new("hello")).expect("Should be OK");
    cable.add_connection(VaryGreet::new("hi")).expect("Should be OK");
    assert_eq!(cable.hello("world"), ["hello world", "hi world"]);
    assert_eq!(cable.hello("nico"), ["hello nico", "hi nico"]);
    assert_eq!(cable.bye("qwq"), ["bye qwq", "bye qwq"]);
    cable.disconnect().expect("Should be OK");
    // Dynamic arrays
    let mut cable = ArrayCable::<&mut dyn Greet>::new();
    let mut greet1 = VaryGreet::new("h1");
    let mut greet2 = VaryGreet::new("h2");
    let mut counted = CountedGreet::new();
    cable.add_connection(&mut greet1).expect("Should be OK");
    cable.add_connection(&mut greet2).expect("Should be OK");
    cable.add_connection(&mut counted).expect("Should be OK");
    assert_eq!(cable.hello("world"), ["h1 world", "h2 world", "hello #0 world"]);
    assert_eq!(cable.hello("nico"), ["h1 nico", "h2 nico", "hello #1 nico"]);
    assert_eq!(cable.bye("qwq"), ["bye qwq", "bye qwq", "bye #2 qwq"]);
    cable.disconnect().expect("Should be OK");
}
