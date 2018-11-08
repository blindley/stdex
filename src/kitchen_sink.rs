
pub trait Increment {
    fn increment(&mut self);
    fn next(mut self) -> Self where Self: Copy {
        self.increment();
        self
    }
}

pub trait Decrement {
    fn decrement(&mut self);
    fn previous(mut self) -> Self where Self: Copy {
        self.decrement();
        self
    }
}

macro_rules! impl_increment_for_integer {
    ($($t:ty)*) => {
        $(
            impl Increment for $t {
                #[inline]
                fn increment(&mut self) { *self += 1; }
            }

            impl Decrement for $t {
                #[inline]
                fn decrement(&mut self) { *self -= 1; }
            }
        )*
    };
}

impl_increment_for_integer! {
    i8 i16 i32 i64 i128 isize
    u8 u16 u32 u64 u128 usize
}