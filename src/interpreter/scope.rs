use super::value::ItpValue;
use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Scope {
    pub parent: Option<Rc<RefCell<Scope>>>,
    pub bindings: HashMap<String, Rc<ItpValue>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            parent: None,
            bindings: HashMap::new(),
        }
    }

    pub fn new_child(parent: Rc<RefCell<Scope>>) -> Self {
        Scope {
            parent: Some(parent),
            bindings: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Rc<ItpValue>> {
        if let Some(value) = self.bindings.get(name) {
            return Some(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get(name);
        }

        None
    }

    pub fn set(&mut self, name: String, value: Rc<ItpValue>) -> Result<()> {
        if self.bindings.contains_key(&name) {
            return Err(anyhow!("Variable already defined"));
        }

        self.bindings.insert(name, value);
        Ok(())
    }

    pub fn replace(&mut self, name: String, value: Rc<ItpValue>) -> Result<()> {
        if self.bindings.contains_key(&name) {
            let ty = self.bindings.get(&name).unwrap().get_type();
            if ty != value.get_type() {
                return Err(anyhow!("Variable already defined with different type"));
            }
            self.bindings.insert(name, value);
            return Ok(());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().replace(name, value);
        }

        Err(anyhow!("Variable not defined"))
    }

    pub fn set_or_replace(&mut self, name: String, value: Rc<ItpValue>) -> Result<()> {
        // first try to replace
        if let Err(e) = self.replace(name.clone(), value.clone()) {
            // if error is because variable is not defined, then set
            if e.to_string().contains("not defined") {
                self.set(name, value)
            } else {
                Err(e)
            }
        } else {
            Ok(())
        }
    }
}
