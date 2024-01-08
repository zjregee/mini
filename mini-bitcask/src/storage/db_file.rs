use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::prelude::FileExt;
use std::path::Path;

use crate::storage::entry;
use crate::utils::hash_routine;

#[derive(Default)]
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
        let key = self.read_buf(offset, entry.key_size);
        if key.is_none() {
            return None;
        }
        entry.key = key.unwrap();
        offset += entry.key_size;
        let value = self.read_buf(offset, entry.value_size);
        if value.is_none() {
            return None;
        }
        entry.value = value.unwrap();
        let check_sum = hash_routine::encode_vec_u8(&entry.value, 32) as u32;
        if check_sum != entry.crc32 {
            return None;
        }
        entry.valid = true;
        Some(entry)
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

    pub fn close(&mut self) -> bool {
        if self.file.is_none() {
            return false;
        }
        if self.file.as_ref().unwrap().sync_all().is_err() {
            return false;
        }
        self.file = None;
        true
    }

    fn read_buf(&self, offset: u32, len: u32) -> Option<Vec<u8>> {
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
}