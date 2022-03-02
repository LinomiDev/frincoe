trait Empty {}

pub trait PubEmpty {}

trait SayHello {
    fn hello(&self, name: &str) -> std::io::Result<()>;
}

mod pathing {
    pub trait MoreFns {
        fn f1(self, x: i32) -> u32;
        fn f2(&self, x: u32) -> i32;
        fn f3(&mut self, x: f32, y: f32) -> (f32, f32);
    }
}
