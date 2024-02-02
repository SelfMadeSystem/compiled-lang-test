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

    pub fn new_temp_name(&mut self) -> String {
        let mut new_name = "temp".to_string();
        let mut i = 0;

        while self.bindings.contains_key(&new_name) {
            i += 1;
            new_name = format!("temp_{}", i);
        }

        self.bindings
            .insert(new_name.clone(), Rc::new(ItpValue::Temp));

        new_name
    }
}
