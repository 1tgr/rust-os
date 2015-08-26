pub use marshal::PackedArgs;

pub trait Dispatch {
    fn dispatch(&self, num: usize, args: PackedArgs) -> isize;
}
