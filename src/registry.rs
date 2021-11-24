use crate::cache::Cache;
#[allow(unused)]
use anyhow::{anyhow, Result};
use bytes::Bytes;
use wasmer::Module;
use wasmer::Store;

#[allow(unused)]
const RUNTIME_URL: &'static str = "https://host.com/runtime.wasm";

#[cfg(not(debug_assertions))]
fn download_runtime() -> Result<Bytes> {
    // TODO: include version
    reqwest::blocking::get(RUNTIME_URL)
        .map_err(|err| anyhow!("Error downloading runtime: {}", err.to_string()))?
        .bytes()
        .map_err(|err| {
            anyhow!(
                "Error readding response when downloading runtime: {}",
                err.to_string()
            )
        })
}

#[cfg(debug_assertions)]
fn download_runtime() -> Result<Bytes, Box<dyn std::error::Error>> {
    // Ok(Bytes::from(include_bytes!("../../target/wasm32-wasi/debug/quickjs_rt.wasm").to_vec()))
    Ok(Bytes::from(Vec::new()))
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
        // TODO: runtime is hardcoded now. Need an index file, uploaded to the server,
        // with a map of version to repositories urls of wasm files etc.
        if self.cache.has_wasm(runtime) {
            Ok(Bytes::from(
                std::fs::read(self.cache.wasm_path(runtime)).unwrap(),
            ))
        } else {
            let bytes = download_runtime().unwrap();
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
            unsafe { Module::deserialize_from_file(&store, mod_path)? }
        } else {
            let wasm_bytes = self.get_wasm(runtime).unwrap().to_vec();
            let module = Module::new(&store, wasm_bytes)?;
            module.serialize_to_file(mod_path)?;
            module
        };
        Ok(module)
    }
}
