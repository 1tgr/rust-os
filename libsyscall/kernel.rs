pub trait Dispatch {
    fn dispatch(&self, num: usize, arg1: usize, arg2: usize) -> usize;
}
