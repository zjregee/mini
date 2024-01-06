use crate::utils::hash_routine;
use crate::utils::time_routine;

pub const ENTRY_HEADER_SIZE: u32 = 26;

#[derive(Clone)]
pub struct Meta {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub extra: Vec<u8>,
    pub key_size: u32,
    pub value_size: u32,
    pub extra_size: u32
}

pub struct Entry {
    pub valid: bool,
    pub meta: Meta,
    pub state: u16,
    pub crc32: u32,
    pub time_stamp: u64,
}

impl Entry {
    pub fn new(key: Vec<u8>, value: Vec<u8>, extra: Vec<u8>, t: u8, mark: u8) -> Entry {
        let mut state = 0;
        state = state | ((t as u16) << 8);
        state = state | (mark as u16);
        Entry {
            valid: true,
            crc32: 0,
            time_stamp: time_routine::time_now(),
            state,
            meta: Meta {
                key_size: key.len() as u32,
                value_size: value.len() as u32,
                extra_size: extra.len() as u32,
                key,
                value,
                extra,
            },
        }
    }

    pub fn new_with_expire(key: Vec<u8>, value: Vec<u8>, extra: Vec<u8>, deadline: u64, t: u16, mark: u16) -> Entry {
        let mut state = 0;
        state = state | ((t as u16) << 8);
        state = state | (mark as u16);
        Entry {
            valid: true,
            crc32: 0,
            time_stamp: deadline,
            state,
            meta: Meta {
                key_size: key.len() as u32,
                value_size: value.len() as u32,
                extra_size: extra.len() as u32,
                key,
                value,
                extra,
            },
        }
    }

    pub fn new_no_extra(key: Vec<u8>, value: Vec<u8>, t: u16, mark: u16) -> Entry {
        let mut state = 0;
        state = state | ((t as u16) << 8);
        state = state | (mark as u16);
        Entry {
            valid: true,
            crc32: 0,
            time_stamp: time_routine::time_now(),
            state,
            meta: Meta {
                extra_size: 0,
                extra: vec![],
                key_size: key.len() as u32,
                value_size: value.len() as u32,
                key,
                value,
            },
        }
    }

    pub fn decode_header(buf: Vec<u8>) -> Option<Entry> {
        if buf.len() < ENTRY_HEADER_SIZE as usize {
            return None;
        }
        let ks = buf[7] as u32 | ((buf[6] as u32) << 8) | ((buf[5] as u32) << 16) | ((buf[4] as u32) << 24);
        let vs = buf[11] as u32 | ((buf[10] as u32) << 8) | ((buf[9] as u32) << 16) | ((buf[8] as u32) << 24);
        let es = buf[15] as u32 | ((buf[14] as u32) << 8) | ((buf[13] as u32) << 16) | ((buf[12] as u32) << 24);
        let state = buf[17] as u16 | ((buf[16] as u16) << 8);
        let time_stamp = buf[25] as u64 | ((buf[24] as u64) << 8) | ((buf[23] as u64) << 16) | ((buf[22] as u64) << 24)
                            | ((buf[21] as u64) << 32) | ((buf[20] as u64) << 40) | ((buf[19] as u64) << 48 | (buf[18] as u64) << 56);
        let crc = buf[3] as u32 | ((buf[2] as u32) << 8) | ((buf[1] as u32) << 16) | ((buf[0] as u32) << 24);
        Some(Entry {
            valid: false,
            crc32: crc,
            state,
            time_stamp,
            meta: Meta {
                key: vec![],
                value: vec![],
                extra: vec![],
                key_size: ks,
                value_size: vs,
                extra_size: es,
            },
        })
    }

    pub fn encode(&self) -> Option<Vec<u8>> {
        if !self.valid {
            return None;
        }
        let ks = self.meta.key_size;
        let vs = self.meta.value_size;
        let es = self.meta.extra_size;
        let state = self.state;
        let time_stamp = self.time_stamp;
        let mut buf = Vec::<u8>::with_capacity(ENTRY_HEADER_SIZE as usize);
        for _ in 0..ENTRY_HEADER_SIZE {
            buf.push(0);
        }
        buf[4] = (ks >> 24) as u8;
        buf[5] = (ks >> 16) as u8;
        buf[6] = (ks >> 8) as u8;
        buf[7] = ks as u8;
        buf[8] = (vs >> 24) as u8;
        buf[9] = (vs >> 16) as u8;
        buf[10] = (vs >> 8) as u8;
        buf[11] = vs as u8;
        buf[12] = (es >> 24) as u8;
        buf[13] = (es >> 16) as u8;
        buf[14] = (es >> 8) as u8;
        buf[15] = es as u8;
        buf[16] = (state >> 8) as u8;
        buf[17] = state as u8;
        buf[18] = (time_stamp >> 56) as u8;
        buf[19] = (time_stamp >> 48) as u8;
        buf[20] = (time_stamp >> 40) as u8;
        buf[21] = (time_stamp >> 32) as u8;
        buf[22] = (time_stamp >> 24) as u8;
        buf[23] = (time_stamp >> 16) as u8;
        buf[24] = (time_stamp >> 8) as u8;
        buf[25] = time_stamp as u8;
        buf.extend_from_slice(&self.meta.key);
        buf.extend_from_slice(&self.meta.value);
        buf.extend_from_slice(&self.meta.extra);
        let check_sum = hash_routine::encode_vec_u8(&self.meta.value, 32) as u32;
        buf[0] = (check_sum >> 24) as u8;
        buf[1] = (check_sum >> 16) as u8;
        buf[2] = (check_sum >> 8) as u8;
        buf[3] = check_sum as u8;
        Some(buf)
    }

    pub fn get_type(&self) -> u16 {
        self.state >> 8
    }

    pub fn get_mark(&self) -> u16 {
        self.state & ((1<<8) - 1)
    }

    pub fn size(&self) -> u32 {
        ENTRY_HEADER_SIZE + self.meta.key_size + self.meta.value_size + self.meta.extra_size
    }
}