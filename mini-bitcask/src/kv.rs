use std::fs;
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;

use crate::config;
use crate::ds::hash;
use crate::storage::entry;
use crate::storage::db_file;
use crate::utils::time_routine;

enum EntryType {
    Set,
    SetWithExpire,
    Delete,
    Clear,
}

impl Into<u16> for EntryType {
    fn into(self) -> u16 {
        match self {
            EntryType::Set => 0,
            EntryType::SetWithExpire => 1,
            EntryType::Delete => 2,
            EntryType::Clear => 3,
        }
    }
}

impl From<u16> for EntryType {
    fn from(kind: u16) -> Self {
        match kind {
            0 => EntryType::Set,
            1 => EntryType::SetWithExpire,
            2 => EntryType::Delete,
            3 => EntryType::Clear,
            _ => panic!(),
        }
    }
}

pub fn check_key_value(key: &String, value: &String) -> bool {
    if key.len() == 0 || key.len() > 100 {
        return false;
    }
    value.len() <= 100
}

pub fn check_key_value_vec(key: &Vec<u8>, value: &Vec<u8>) -> bool {
    if key.len() == 0 || key.len() > 100 {
        return false;
    }
    value.len() <= 100
}

pub fn build(path: &String) -> Option<Vec<u32>> {
    let dir =  match fs::read_dir(path) {
        Ok(dir) => dir,
        Err(_) => return None,
    };
    let mut ids = vec![];
    for entry in dir {
        let file_name = entry.ok()?.file_name().to_str()?.to_string();
        if file_name.contains(".data") {
            let id = file_name.split(".").next()?.parse::<u32>().ok()?;
            ids.push(id);
        }
    }
    Some(ids)
}

#[derive(Default)]
#[allow(non_camel_case_types)]
pub struct kv {
    pub config: config::Config,
    pub hash_index: hash::Hash,
    pub expires: HashMap<String, u64>,
    pub active_file: db_file::DBFile,
    pub arch_files: HashMap<u32, db_file::DBFile>,
}

impl kv {
    pub fn open(config: config::Config) -> Option<kv> {
        fs::create_dir(&config.dir_path).ok()?;
        let mut ids = build(&config.dir_path)?;
        ids.sort();
        let mut arch_files = HashMap::default();
        let active_file = if ids.len() == 0 {
            let active_id = 1;
            let active_path = Path::new(&config.dir_path).join(format!("{}.data", active_id));
            File::create(active_path).expect("active file can't create error");
            db_file::DBFile::new(config.dir_path.clone(), active_id).expect("active file can't create error")
        } else {
            for i in 0..ids.len() - 1 {
                arch_files.insert(ids[i], db_file::DBFile::new(config.dir_path.clone(), ids[i]).expect("archive file can't open error"));
            }
            db_file::DBFile::new(config.dir_path.clone(), ids[ids.len() - 1]).expect("active file can't open error")
        };
        let mut db = kv {
            config,
            hash_index: hash::Hash::default(),
            expires: HashMap::default(),
            active_file,
            arch_files,
        };
        db.build_index();
        Some(db)
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
        self.hash_index.set(key.clone(), value.clone());
        let entry = entry::Entry::new(key.into_bytes(), value.into_bytes(), 0, EntryType::Set.into());
        self.store_entry(entry)
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
        self.hash_index.set(key.clone(), value.clone());
        let entry = entry::Entry::new_with_expire(key.into_bytes(), value.into_bytes(), deadline, 0, EntryType::SetWithExpire.into());
        self.store_entry(entry)
    }

    pub fn delete(&mut self, key: String) -> bool {
        if !check_key_value(&key, &"".to_string()) {
            return false;
        }
        if self.expires.contains_key(&key) {
            self.expires.remove(&key);
        }
        self.hash_index.delete(key.clone());
        let entry = entry::Entry::new(key.into_bytes(), vec![], 0, EntryType::Delete.into());
        self.store_entry(entry)
    }

    pub fn clear(&mut self) -> bool {
        self.expires.clear();
        self.hash_index.clear();
        let entry = entry::Entry::new(vec![], vec![], 0, EntryType::Clear.into());
        self.store_entry(entry)
    }

    pub fn close(&mut self) {
        for (_, arch_file) in self.arch_files.iter_mut() {
            arch_file.close();
        }
        self.active_file.close();
    }

    pub fn check_expired(&self, key: &String) -> bool {
        if !self.expires.contains_key(key) {
            return true;
        }
        let deadline = self.expires[key];
        time_routine::time_now() <= deadline
    }

    pub fn store_entry(&mut self, entry: entry::Entry) -> bool {
        if self.active_file.offset > self.config.max_file_size {
            let active_id = self.active_file.id;
            if !self.active_file.close() {
                return false;
            }
            let new_id = active_id + 1;
            let new_path = Path::new(&self.config.dir_path).join(format!("{}.data", new_id));
            if File::create(new_path).is_err() {
                return false;
            }
            self.active_file = db_file::DBFile::new(self.config.dir_path.clone(), new_id).expect("active file can't create error");
            self.arch_files.insert(active_id, db_file::DBFile::new(self.config.dir_path.clone(), active_id).expect("active file can't archive error"));
        }
        self.active_file.write(entry)
    }

    fn build_index(&mut self) {
        if self.arch_files.len() > 0 {
            let mut ids = vec![];
            for id in self.arch_files.keys() {
                ids.push(*id);
            }
            ids.sort();
            for id in ids {
                let mut offset = 0;
                loop {
                    let entry = self.arch_files.get(&id).unwrap().read(offset);
                    if entry.is_none() {
                        break;
                    }
                    let entry = entry.unwrap();
                    offset += entry.size();
                    if check_key_value_vec(&entry.key, &entry.value) {
                        self.build_entry(entry);
                    }
                }
            }
        }
        let mut offset = 0;
        loop {
            let entry = self.active_file.read(offset);
            if entry.is_none() {
                break;
            }
            let entry = entry.unwrap();
            offset += entry.size();
            if check_key_value_vec(&entry.key, &entry.value) {
                self.build_entry(entry);
            }
        }
    }

    fn build_entry(&mut self, entry: entry::Entry) {
        if !entry.valid {
            return;
        }
        match EntryType::from(entry.get_mark()) {
            EntryType::Set => {
                let key = match String::from_utf8(entry.key) {
                    Ok(key) => key,
                    Err(_) => return,
                };
                let value: String = match String::from_utf8(entry.value) {
                    Ok(value) => value,
                    Err(_) => return,
                };
                self.hash_index.set(key, value);
            },
            EntryType::SetWithExpire => {
                let key = match String::from_utf8(entry.key) {
                    Ok(key) => key,
                    Err(_) => return,
                };
                let value: String = match String::from_utf8(entry.value) {
                    Ok(value) => value,
                    Err(_) => return,
                };
                if entry.time_stamp > time_routine::time_now() {
                    if self.expires.contains_key(&key) {
                        *self.expires.get_mut(&key).unwrap() = entry.time_stamp;
                    } else {
                        self.expires.insert(key.clone(), entry.time_stamp);
                    }
                }
                self.hash_index.set(key, value);
            },
            EntryType::Delete => {
                let key = match String::from_utf8(entry.key) {
                    Ok(key) => key,
                    Err(_) => return,
                };
                self.hash_index.delete(key);
            },
            EntryType::Clear => {
                self.hash_index.clear();
            },
        }
    }
}