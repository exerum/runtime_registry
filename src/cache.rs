use std::path::PathBuf;
use anyhow::{Result, anyhow};

pub fn satru_home() -> Result<PathBuf> {
    match std::env::var("SATRU_HOME") {
        Ok(home) => Ok(PathBuf::from(home)),
        Err(_) => {
            let mut user_home = dirs::home_dir()
                .ok_or(anyhow!("Could not get user's home directory"))?;
            user_home.push("satru-cli");
            Ok(user_home)
        }
    }
}

pub struct Cache {
    root: PathBuf
}

impl Cache {
    pub fn new() -> Self {
        let mut cache_dir = satru_home().unwrap();
        cache_dir.push("cache");
        Cache {
            root: cache_dir
        }
    }

    pub fn clear(&self) {
        println!("Root is: {:?}", &self.root);
        std::fs::remove_dir_all(&self.root).unwrap();
        std::fs::create_dir_all(&self.root).unwrap();
    }

    pub fn put_wasm(&self, runtime: &str, data: &Vec<u8>) {
        let path = self.wasm_path(runtime);
        let rt_path = path.parent().unwrap();
        std::fs::create_dir_all(&rt_path).unwrap();
        std::fs::write(path, data).unwrap();
    }

    pub fn has_wasm(&self, runtime: &str) -> bool {
        self.wasm_path(runtime).exists()
    }

    pub fn wasm_path(&self, runtime: &str) -> PathBuf {
        let mut path = self.root.clone();
        path.push(runtime);
        path.push("index.wasm");
        path
    }

    /// Compiled wasm module's path in file system
    pub fn module_path(&self, runtime: &str) -> PathBuf {
        let mut path = self.root.clone();
        path.push(runtime);
        path.push("index.mod");
        path
    }

    pub fn has_compiled_module(&self, runtime: &str) -> bool {
        self.module_path(runtime).exists()
    }

    #[allow(dead_code)]
    pub fn get_compiled_module(&self, runtime: &str) -> Option<Result<Vec<u8>, std::io::Error>> {
        let path = self.module_path(runtime);
        match path.exists() {
            true => {
                Some(std::fs::read(path))
            },
            false => {
                None
            }
        }
    }
}