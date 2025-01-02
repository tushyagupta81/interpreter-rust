use crate::expr::LiteralValue;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Environments {
    values: HashMap<String, LiteralValue>,
    pub enclosing: Option<Rc<RefCell<Environments>>>,
}

impl Environments {
    pub fn new() -> Self {
        Environments {
            values: HashMap::<String, LiteralValue>::new(),
            enclosing: None,
        }
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &str, value: LiteralValue) -> bool {
        let old_value = self.values.get(name);

        match (old_value, &self.enclosing) {
            (Some(_), _) => {
                self.values.insert(name.to_string(), value);
                true
            }
            (None, Some(env)) => env.borrow_mut().assign(name, value),
            (None, None) => false,
        }
    }

    pub fn get(&self, name: &str) -> Option<LiteralValue> {
        let val = self.values.get(name);

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
        let _env = Environments::new();
    }
}
