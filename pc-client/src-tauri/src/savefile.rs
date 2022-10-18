use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub struct SaveFile {
    path: String,
    data: SaveData,
}

#[derive(Serialize, Deserialize)]
struct SaveData {
    host: String,
}

impl SaveFile {
    pub fn new(file_path: &str) -> SaveFile {
        // let path = env::current_dir().unwrap();
        // let dir = path.as_path().read_dir().unwrap();

        if !Path::new(&file_path.to_string()).is_file() {
            // Open a file in write-only mode, returns `io::Result<File>`
            let _ = File::create(&file_path);
            SaveFile {
                path: "".to_string(),
                data: SaveData {
                    host: "".to_string(),
                },
            }
        } else {
            let f_str = fs::read_to_string(file_path).unwrap();

            if let Ok(save_data) = serde_yaml::from_str::<SaveData>(&f_str) {
                SaveFile {
                    path: file_path.to_string(),
                    data: save_data,
                }
            } else {
                SaveFile {
                    path: "".to_string(),
                    data: SaveData {
                        host: "".to_string(),
                    },
                }
            }
        }
    }

    fn save_data_to_file(&self) {
        let str = serde_yaml::to_string(&self.data).unwrap();

        let mut options = OpenOptions::new();
        if let Ok(mut file) = options.write(true).open(&self.path){

            let _ = file.write(str.as_bytes());
        }
    }

    pub fn get_host(&self) -> String {
        self.data.host.clone()
    }

    pub fn set_host(&mut self, key: &str) {
        self.data.host = key.to_string();
        self.save_data_to_file();
    }
}
