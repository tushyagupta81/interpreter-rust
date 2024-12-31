use crate::expr::LiteralValue;
use std::{collections::HashMap, error::Error};

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

    pub fn assign(&mut self, name: String, value: LiteralValue) -> Result<LiteralValue, Box<dyn Error>> {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.values.entry(name.clone()) {
            e.insert(value.clone());
        } else {
            return Err(format!("Undefined variable {}", name).into());
        }
        Ok(value)
    }

    pub fn get(&self, name: &str) -> Option<&LiteralValue> {
        self.values.get(name)
    }
}
