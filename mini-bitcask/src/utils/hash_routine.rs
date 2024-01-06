use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub fn encode_vec_u8(input: &[u8], size: usize) -> u64 {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish() & ((1 << size) - 1)
}