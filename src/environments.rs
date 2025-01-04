use crate::expr::LiteralValue;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

// The Environment holds all the variables and their values if any and also holds a reference to a
// parent Environment if any
pub struct Environment {
    values: HashMap<String, LiteralValue>,
    // Enclosing is the parent Environment to the current Environment
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::<String, LiteralValue>::new(),
            enclosing: None,
        }
    }

    // create a new variable or override a existing variable of same name
    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    // Assign a value to a already existing variable
    pub fn assign(&mut self, name: &str, value: LiteralValue) -> bool {
        let old_value = self.values.get(name);

        match (old_value, &self.enclosing) {
            // Check if variable exists in current Environment
            (Some(_), _) => {
                self.values.insert(name.to_string(), value);
                true
            }
            // Check if variable exists in parent Environment recurcively
            (None, Some(env)) => env.borrow_mut().assign(name, value),
            // Variable was never declared
            (None, None) => false,
        }
    }

    // Get the value of a variable
    pub fn get(&self, name: &str) -> Option<LiteralValue> {
        let val = self.values.get(name);

        // Get the value of a variable even if it was declared in parent Environments
        match (val, &self.enclosing) {
            (Some(v), _) => Some(v.clone()),
            (None, Some(env)) => env.borrow().get(name),
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_init() {
        let _env = Environment::new();
    }
}
