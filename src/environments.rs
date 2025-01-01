use crate::expr::LiteralValue;
use std::{collections::HashMap, error::Error, rc::Rc};

pub struct Environments {
    values: HashMap<String, LiteralValue>,
    pub enclosing: Option<Rc<Environments>>,
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

    pub fn assign(
        &mut self,
        name: String,
        value: LiteralValue,
    ) -> Result<LiteralValue, Box<dyn Error>> {
        if let Some(en) = &mut self.enclosing {
            return Rc::get_mut(en).expect("failed to get mut reference to enclosing").assign(name, value);
        }

        if let std::collections::hash_map::Entry::Occupied(mut e) = self.values.entry(name.clone())
        {
            e.insert(value.clone());
        } else {
            return Err(format!("Undefined variable {}", name).into());
        }
        Ok(value)
    }

    pub fn get(&self, name: &str) -> Option<&LiteralValue> {
        let val = self.values.get(name);

        match (val, &self.enclosing) {
            (Some(v), _) => Some(v),
            (None, Some(env)) => env.get(name),
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
