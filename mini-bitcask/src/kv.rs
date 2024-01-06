use std::collections::HashMap;
use serde::de;

use crate::config;
use crate::ds::hash;
use crate::utils::time_routine;

pub fn check_key_value(key: &String, value: &String) -> bool {
    if key.len() == 0 || key.len() > 100 {
        return false;
    }
    value.len() <= 100
}

#[derive(Default)]
pub struct kv {
    pub hash_index: hash::Hash,
    pub expires: HashMap<String, u64>,
}

impl kv {
    pub fn open(config: config::Config) -> Option<kv> {
        Some(kv::default())
    }

    pub fn get(&mut self, key: String) -> Option<String> {
        if !check_key_value(&key, &"".to_string()) {
            return None;
        }
        if !self.check_expired(&key) {
            self.hash_index.delete(key);
            return None
        }
        self.hash_index.get(key)
    }

    pub fn set(&mut self, key: String, value: String) -> bool {
        if !check_key_value(&key, &value) {
            return false;
        }
        self.hash_index.set(key, value);
        true
    }

    pub fn set_with_expire(&mut self, key: String, value: String, deadline: u64) -> bool {
        if !check_key_value(&key, &value) {
            return false;
        }
        if self.expires.contains_key(&key) {
            *self.expires.get_mut(&key).unwrap() = deadline;
        } else {
            self.expires.insert(key.clone(), deadline);
        }
        self.hash_index.set(key, value);
        true
    }

    pub fn delete(&mut self, key: String) -> bool {
        if !check_key_value(&key, &"".to_string()) {
            return false;
        }
        if self.expires.contains_key(&key) {
            self.expires.remove(&key);
        }
        self.hash_index.delete(key);
        true
    }

    pub fn clear(&mut self) -> bool {
        self.expires.clear();
        self.hash_index.clear();
        true
    }

    pub fn check_expired(&self, key: &String) -> bool {
        if !self.expires.contains_key(key) {
            return true;
        }
        let deadline = self.expires[key];
        time_routine::time_now() <= deadline
    }
}