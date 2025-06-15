use std::collections::HashMap;

use crate::tokens::Object;

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: HashMap<String, Object>,
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
}
