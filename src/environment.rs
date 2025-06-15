use std::collections::HashMap;

use crate::interpreter::RuntimeError;
use crate::tokens::{Object, Token};

pub struct EnvironmentStack {
    environments: Vec<HashMap<String, Object>>,
}

impl EnvironmentStack {
    pub fn new() -> Self {
        EnvironmentStack {
            environments: vec![HashMap::new()],
        }
    }

    pub fn current_environment(&mut self) -> &mut HashMap<String, Object> {
        self.environments.last_mut().unwrap()
    }

    pub fn push_environment(&mut self) {
        self.environments.push(HashMap::new());
    }

    pub fn pop_environment(&mut self) {
        if self.environments.len() > 1 {
            self.environments.pop();
        }
    }

    pub fn define(&mut self, name: &Token, value: Object) {
        self.current_environment()
            .insert(name.lexeme.clone(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, RuntimeError> {
        // Search through the stack from top to bottom (most recent to oldest)
        for environment in self.environments.iter().rev() {
            if let Some(value) = environment.get(&name.lexeme) {
                return Ok(value.clone());
            }
        }

        Err(RuntimeError {
            message: format!("Undefined variable '{}'.", name.lexeme),
            token: name.clone(),
        })
    }

    pub fn define_global(&mut self, name: &str, value: Object) {
        self.environments
            .first_mut()
            .unwrap()
            .insert(name.to_owned(), value);
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), RuntimeError> {
        // Search through the stack from top to bottom (most recent to oldest)
        for environment in self.environments.iter_mut().rev() {
            if environment.contains_key(&name.lexeme) {
                environment.insert(name.lexeme.clone(), value);
                return Ok(());
            }
        }

        Err(RuntimeError {
            message: format!("Undefined variable '{}'.", name.lexeme),
            token: name.clone(),
        })
    }
}
