use crate::field::*;
use crate::FungeRenderer;

pub struct FungeError(String);

pub trait FungeStack<T> {
    fn push(&mut self, value: T);
    fn pop(&mut self) -> T;
}

impl FungeStack<i32> for Vec<i32> {
    fn push(&mut self, value: i32) {
        self.push(value);
    }
    fn pop(&mut self) -> i32 {
        self.pop().unwrap_or_default()
    }
}

enum Renderable<'a> {
    Field(&'a FungeField),
    Stack(&'a dyn FungeStack<i32>),

}

pub struct FungeEnvironment<T> {
    field: FungeField,
    renderer: Box<dyn FungeRenderer>,
    stack: FungeStack<T>
}

pub trait Operation<T> {
    fn execute(env: &FungeEnvironment<T>) -> Result<&FungeEnvironment<T>, FungeError>;
}

pub struct Discard;
impl<T> Operation<T> for Discard {
    fn execute(env: &FungeEnvironment<T>) -> Result<&FungeEnvironment<T>, FungeError> {
        Ok(env)
    }
}
