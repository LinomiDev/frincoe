mod hello_mod {
    pub trait Hello {
        fn hello(&self, name: &str) -> String {
            format!("hello {}", name)
        }
    }
}

mod provider_mod {
    pub struct HelloProvider {
        pub word: String,
    }

    impl super::hello_mod::Hello for HelloProvider {
        fn hello(&self, name: &str) -> String {
            format!("{} {}", self.word, name)
        }
    }
}

mod client_mod {
    use frincoe_macros::{forward_sub, inject_implement};
    pub struct HelloClient {
        pub data: super::provider_mod::HelloProvider,
    }

    inject_implement! {
        impl "read_trait_from_file.rs"::hello_mod::Hello as super::hello_mod::Hello
            for HelloClient in forward_sub(data)
    }
}

fn main() {
    use client_mod::HelloClient;
    use hello_mod::Hello;
    use provider_mod::HelloProvider;
    let client = HelloClient {
        data: HelloProvider { word: "hi".to_string() },
    };
    assert_eq!(client.hello("world"), "hi world");
}
