use std::collections::HashMap;

#[derive(Default)]
pub struct Hash {
    index: HashMap<String, String>,
}

impl Hash {
    pub fn get(&self, key: String) -> Option<String> {
        if self.index.contains_key(&key) {
            self.index.get(&key).cloned()
        } else {
            None
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        if self.index.contains_key(&key) {
            *self.index.get_mut(&key).unwrap() = value;
        } else {
            self.index.insert(key, value);
        }
    }

    pub fn delete(&mut self, key: String) {
        if self.index.contains_key(&key) {
            self.index.remove(&key);
        }
    }

    pub fn clear(&mut self) {
        self.index.clear();
    }
}