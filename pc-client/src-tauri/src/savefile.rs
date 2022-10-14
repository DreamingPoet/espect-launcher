use std::fs::{self, File};
use std::path::Path;

pub struct SaveFile {
    path: String,
}

impl SaveFile {
    pub fn new(file_path: &str) -> SaveFile {
        // let path = env::current_dir().unwrap();
        // let dir = path.as_path().read_dir().unwrap();
        
        if !Path::new(&file_path.to_string()).is_file() {
            // Open a file in write-only mode, returns `io::Result<File>`
            let _ = File::create(&file_path);
        }
        
        SaveFile {
            path: file_path.to_string(),
        }
    }

    pub fn get_value(&self, key: &str) -> String{

        String::from(" ")
    }

    pub fn set_value(&self, key: &str) {
        
    }
}
