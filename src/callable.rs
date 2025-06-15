use crate::{
    interpreter::{Interpreter, RuntimeError},
    tokens::Object,
};

pub trait Callable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<Object>,
    ) -> Result<Object, RuntimeError>;

    fn arity(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeCallable {
    arity: usize,
    func: fn(&mut Interpreter, Vec<Object>) -> Result<Object, RuntimeError>,
}

impl NativeCallable {
    pub fn new(
        arity: usize,
        func: fn(&mut Interpreter, Vec<Object>) -> Result<Object, RuntimeError>,
    ) -> Self {
        Self { arity, func }
    }
}

impl Callable for NativeCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: Vec<Object>,
    ) -> Result<Object, RuntimeError> {
        (self.func)(interpreter, args)
    }

    fn arity(&self) -> usize {
        self.arity
    }
}
