use std::collections::HashMap;

use crate::{
    interpreter::RuntimeError,
    tokens::{Object, Token},
};

#[derive(Debug, Clone)]
pub struct Environment {
    pub enclosing: Option<Box<Environment>>,
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new(enclosing: Option<Box<Environment>>) -> Self {
        Environment {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, RuntimeError> {
        let not_found = RuntimeError {
            message: format!("Undefined variable '{}'.", name.lexeme),
            token: name.clone(),
        };

        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => match &self.enclosing {
                Some(enclosing) => match enclosing.get(name) {
                    Ok(value) => Ok(value.clone()),
                    Err(_) => Err(not_found),
                },
                None => Err(not_found),
            },
        }
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), RuntimeError> {
        let not_found = RuntimeError {
            message: format!("Undefined variable '{}'.", name.lexeme),
            token: name.clone(),
        };
        if let Some(existing_value) = self.values.get_mut(&name.lexeme) {
            *existing_value = value;
            return Ok(());
        }
        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.assign(name, value);
        }
        Err(not_found)
    }
}
