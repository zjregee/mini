use std::env;

#[derive(Default)]
pub struct Config {
    pub dir_path: String,
    pub max_file_size: u32,
}

pub fn default_config() -> Config {
    let current_path = env::current_dir().ok().unwrap();
    let temp_dir = current_path.join("data");
    Config {
        dir_path: temp_dir.to_str().unwrap().to_string(),
        max_file_size: 16 * 1024 * 1024,
    }  
}