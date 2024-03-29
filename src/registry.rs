use crate::cache::Cache;
#[allow(unused)]
use anyhow::{anyhow, Result};
use bytes::Bytes;
use wasmer::Module;
use wasmer::Store;

fn make_rt_url(version: &str) -> String {
    format!("https://github.com/exerum/js-runtime/releases/download/v{}/js-runtime.wasm", version)
}

fn download_runtime(runtime: &str) -> anyhow::Result<Bytes> {
    let mut parts = runtime.split("@");
    let _lang = parts.next().unwrap();
    // TODO: support other languages
    let version = parts.next().unwrap();
    reqwest::blocking::get(make_rt_url(version))
        .map_err(|err| anyhow!("Error downloading runtime: {}", err.to_string()))?
        .bytes()
        .map_err(|err| {
            anyhow!(
                "Error readding response when downloading runtime: {}",
                err.to_string()
            )
        })
}

pub struct RuntimeRegistry {
    cache: Cache,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        RuntimeRegistry {
            cache: Cache::new(),
        }
    }

    pub fn get_wasm(&self, runtime: &str) -> Result<Bytes, Box<dyn std::error::Error>> {
        if self.cache.has_wasm(runtime) {
            Ok(Bytes::from(
                std::fs::read(self.cache.wasm_path(runtime)).unwrap(),
            ))
        } else {
            let bytes = download_runtime(runtime).unwrap();
            self.cache.put_wasm(runtime, &bytes.to_vec());
            Ok(bytes)
        }
    }

    pub fn get_module(
        &self,
        runtime: &str,
        store: &Store,
    ) -> Result<Module, Box<dyn std::error::Error>> {
        let mod_path = self.cache.module_path(runtime);
        let module = if self.cache.has_compiled_module(runtime) {
            println!("Cached module: {:?}", mod_path);
            unsafe { Module::deserialize_from_file(&store, mod_path)? }
        } else {
            let wasm_bytes = self.get_wasm(runtime).unwrap().to_vec();
            println!("Downloaded wasm. Length is: {:?}", wasm_bytes.len());
            let module = Module::new(&store, wasm_bytes)?;
            module.serialize_to_file(mod_path)?;
            module
        };
        Ok(module)
    }
}
