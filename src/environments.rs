use crate::expr::LiteralValue;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

// The Environment holds all the variables and their values if any and also holds a reference to a
// parent Environment if any
pub struct Environment {
    values: HashMap<String, LiteralValue>,
    // Enclosing is the parent Environment to the current Environment
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    globals: HashMap<String, LiteralValue>,
}

#[allow(clippy::ptr_arg)]
fn clock_impl(_args: &Vec<LiteralValue>) -> LiteralValue {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Could not get system time")
        .as_millis();
    LiteralValue::Number(now as f64 / 1000.0)
}

fn get_globals() -> HashMap<String, LiteralValue> {
    let mut env = HashMap::new();
    env.insert(
        "clock".to_string(),
        LiteralValue::Callable {
            name: "clock".to_string(),
            arity: 0,
            fun: Rc::new(clock_impl),
        },
    );
    env
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::<String, LiteralValue>::new(),
            globals: get_globals(),
            enclosing: None,
        }
    }

    // create a new variable or override a existing variable of same name
    pub fn define(&mut self, name: String, value: LiteralValue, distance: Option<usize>) {
        if distance.is_none() {
            self.globals.insert(name, value);
        } else {
            let distance = distance.unwrap();
            if distance == 0 {
                self.values.insert(name, value);
            } else {
                self.define(name, value, Some(distance - 1));
            }
        }
    }

    // Assign a value to a already existing variable
    pub fn assign(&mut self, name: &str, value: LiteralValue, distance: Option<usize>) -> bool {
        if distance.is_none() {
            self.globals.insert(name.to_string(), value);
            true
        } else {
            let distance = distance.unwrap();
            if distance == 0 {
                self.values.insert(name.to_string(), value.clone());
                true
            } else {
                match &self.enclosing {
                    None => panic!(
                        "Tried to assign a var that was defined deeper than the current env depth"
                    ),
                    Some(env) => return env.borrow_mut().assign(name, value, Some(distance - 1)),
                }
            }
        }
    }

    // Get the value of a variable
    pub fn get(&self, name: &str, distance: Option<usize>) -> Option<LiteralValue> {
        if distance.is_none() {
            self.globals.get(name).cloned()
        } else {
            let distance = distance.unwrap();
            if distance == 0 {
                self.values.get(name).cloned()
            } else {
                match &self.enclosing {
                    None => panic!(
                        "Tried to resolve a var that was defined deeper than the current env depth"
                    ),
                    Some(env) => env.borrow().get(name, Some(distance - 1)),
                }
            }
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
