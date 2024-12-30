use crate::expr::LiteralValue;
use std::collections::HashMap;

pub struct Environments {
    values: HashMap<String, LiteralValue>,
}

impl Environments {
    pub fn new() -> Self {
        Environments {
            values: HashMap::<String, LiteralValue>::new(),
        }
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name:&str) -> Option<&LiteralValue> {
        self.values.get(name)
    }
}
