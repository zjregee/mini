use std::collections::HashMap;
use std::fs::{File, OpenOptions, self};
use std::io::Write;
use std::os::unix::prelude::FileExt;
use std::path::Path;

use crate::storage::entry;
use crate::utils::hash_routine;

pub struct DBFile {
    pub id: u32,
    pub path: String,
    pub offset: u32,
    pub file: Option<File>
}

impl DBFile {
    pub fn new(path: String, file_id: u32) -> Option<DBFile> {
        let file_path = Path::new(&path).join(format!("{}.data", file_id));
        let ret = OpenOptions::new().read(true).write(true).open(file_path);
        if ret.is_err() {
            return None;
        }
        let f = ret.ok().unwrap();
        Some(DBFile {
            id: file_id,
            offset: f.metadata().unwrap().len() as u32,
            file: Some(f),
            path,
        })
    }

    pub fn read(&self, mut offset: u32) -> Option<entry::Entry> {
        let buf = self.read_buf(offset, entry::ENTRY_HEADER_SIZE);
        if buf.is_none() {
            return None;
        }
        let buf = buf.unwrap();
        let mut entry = entry::Entry::decode_header(buf).unwrap();
        offset += entry::ENTRY_HEADER_SIZE;
        let key = self.read_buf(offset, entry.meta.key_size);
        if key.is_none() {
            return None;
        }
        entry.meta.key = key.unwrap();
        offset += entry.meta.key_size;
        let value = self.read_buf(offset, entry.meta.value_size);
        if value.is_none() {
            return None;
        }
        entry.meta.value = value.unwrap();
        offset += entry.meta.value_size;
        let extra = self.read_buf(offset, entry.meta.extra_size);
        if extra.is_none() {
            return None;
        }
        entry.meta.extra = extra.unwrap();
        let check_sum = hash_routine::encode_vec_u8(&entry.meta.value, 32) as u32;
        if check_sum != entry.crc32 {
            return None;
        }
        entry.valid = true;
        Some(entry)
    }

    pub fn read_buf(&self, offset: u32, len: u32) -> Option<Vec<u8>> {
        if self.file.is_none() {
            return None;
        }
        if offset as u64 >= self.file.as_ref().unwrap().metadata().unwrap().len() {
            return None;
        }
        let mut buf = Vec::with_capacity(len as usize);
        for _ in 0..len {
            buf.push(0);
        }
        let ret = self.file.as_ref().unwrap().read_at(&mut buf, offset as u64);
        if ret.is_err() {
            return None;
        }
        Some(buf)
    }

    pub fn write(&mut self, entry: entry::Entry) -> bool {
        if self.file.is_none() {
            return false;
        }
        if !entry.valid {
            return false;
        }
        let buf = entry.encode();
        if buf.is_none() {
            return false;
        }
        let buf = buf.unwrap();
        let ret = self.file.as_ref().unwrap().write_all(&buf);
        if ret.is_err() {
            return false;
        }
        self.offset += entry.size();
        true
    }

    pub fn close(&mut self, sync: bool) -> bool {
        if self.file.is_none() {
            return true;
        }
        let mut ret = true;
        if sync {
            ret = self.sync();
        }
        self.file = None;
        ret
    }

    pub fn sync(&mut self) -> bool {
        if self.file.is_none() {
            return true;
        }
        let ret = self.file.as_ref().unwrap().sync_all();
        ret.is_ok()
    }
}

pub fn build(path: String) -> Option<(HashMap<u32, DBFile>, u32)> {
    let dir = fs::read_dir(path.clone());
    if dir.is_err() {
        return None;
    }
    let dir = dir.ok().unwrap();
    let mut all_id = vec![];
    for entry in dir {
        let file_name = entry.ok().unwrap().file_name().to_str().unwrap().to_string();
        if file_name.contains(".data") {
            let id = file_name.split(".").next().unwrap().parse::<u32>().ok().unwrap();
            all_id.push(id);
        }
    }
    let mut active_id = 0;
    let mut arch_files = HashMap::new();
    if all_id.len() > 0 {
        all_id.sort();
        active_id = *all_id.last().unwrap();
        for index in 0..all_id.len() - 1 {
            let file = DBFile::new(path.clone(), all_id[index]);
            if file.is_none() {
                return None;
            }
            arch_files.insert(all_id[index], file.unwrap());
        }
    }
    Some((arch_files, active_id))
}