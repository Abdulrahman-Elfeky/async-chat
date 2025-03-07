use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::group::Group;

pub struct GroupTable(Mutex<HashMap<Arc<String>, Arc<Group>>>);

impl GroupTable {
    pub fn new() -> GroupTable {
        GroupTable(Mutex::new(HashMap::new()))
    }
    pub fn get(&self, name: Arc<String>) -> Option<Arc<Group>> {
        self.0.lock().unwrap().get(&name).cloned()
    }
    pub fn get_or_create(&self, name: Arc<String>) -> Arc<Group> {
        self.0
            .lock()
            .unwrap()
            .entry(name.clone())
            .or_insert_with(|| Arc::new(Group::new(name)))
            .clone()
    }
}
