use std::collections::HashMap;

use crate::{
    interpreter::RuntimeError,
    tokens::{Object, Token},
};

pub struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, RuntimeError> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => Err(RuntimeError {
                message: format!("Undefined variable '{}'.", name.lexeme),
                token: name.clone(),
            }),
        }
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            Ok(())
        } else {
            Err(RuntimeError {
                message: format!("Undefined variable '{}'.", name.lexeme),
                token: name.clone(),
            })
        }
    }
}
