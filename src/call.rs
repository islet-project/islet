use crate::communication::Event;

pub struct Context<T: Default + Eq, A: Default> {
    code: T,
    argument: A,
}

impl<T, A> Context<T, A>
where
    T: Default + Eq,
    A: Default,
{
    pub fn argument(&self) -> &A {
        &self.argument
    }

    pub const fn new(code: T, argument: A) -> Self {
        Self { code, argument }
    }
}

impl<T, A> Event for Context<T, A>
where
    T: Default + Copy + Eq,
    A: Default,
{
    type Code = T;

    fn code(&self) -> Self::Code {
        self.code
    }
}
