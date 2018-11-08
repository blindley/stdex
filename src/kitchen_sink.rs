
pub trait Increment {
    fn increment(&mut self);
    fn next(mut self) -> Self where Self: Copy {
        self.increment();
        self
    }
}