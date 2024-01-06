use std::time::SystemTime;

pub fn time_now() -> u64 {
    let time = SystemTime::now();
    time.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}