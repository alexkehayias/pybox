use anyhow::{anyhow, Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use wasmtime::{Cache, Config, Engine, Store, Strategy};
use wasmtime::component::{Component, Linker};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView};

// Default timeout in seconds
const DEFAULT_TIMEOUT_SECONDS: u64 = 40;
const EPOCH_DEADLINE_BASE: u64 = 1; // Additional epoch deadline buffer

struct MyWasi {
    wasi_ctx: WasiCtx,
    table: ResourceTable,
}

impl wasmtime_wasi::WasiView for MyWasi {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.table,
        }
    }
}

// NOTE: This generates a `Sandbox` type automatically and your IDE
// might not pick it up
wasmtime::component::bindgen!({
    path: "sandbox.wit",
    world: "sandbox",
});

/// A sandboxed Python execution environment using WebAssembly.
pub struct PySandbox {
    engine: Engine,
    component: Component,
    pub timeout_seconds: u64,
}

impl PySandbox {
    /// Create a new webassembly sandbox for executing untrusted python code.
    ///
    /// # Arguments
    /// * `timeout_secs` - Optional timeout in seconds. Defaults to `DEFAULT_TIMEOUT_SECONDS`.
    pub fn new(timeout_secs: Option<u64>) -> Result<Self> {
        let timeout_seconds = timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECONDS);

        let mut cfg = Config::new();
        // Enable timeouts
        cfg.epoch_interruption(true);
        // Enable the compilation cache, using the default cache configuration
        // settings.
        cfg.cache(Some(Cache::from_file(None)?));
        let engine = Engine::new(&cfg).expect("Failed to create wasm engine");

        let component = Component::from_file(&engine, "sandbox.wasm")
            .context("Failed to load sandbox.wasm")?;

        Ok(Self {
            engine,
            component,
            timeout_seconds,
        })
    }

    /// Create a sandbox with fast compilation settings for tests.
    ///
    /// Uses Winch baseline compiler, parallel compilation, and caching
    /// to significantly speed up test execution.
    ///
    /// # Arguments
    /// * `timeout_secs` - Optional timeout in seconds. Defaults to `DEFAULT_TIMEOUT_SECONDS`.
    #[allow(dead_code)]
    pub fn new_for_test(timeout_secs: Option<u64>) -> Result<Self> {
        let timeout_seconds = timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECONDS);

        let mut config = Config::new();

        // Enable the compilation cache, using the default cache configuration
        // settings.
        config.cache(Some(Cache::from_file(None)?));

        // Enable Winch, Wasmtime's baseline compiler.
        config.strategy(Strategy::Winch);

        // Enable parallel compilation.
        config.parallel_compilation(true);

        // Enable epoch interruption for timeout support
        config.epoch_interruption(true);

        let engine = Engine::new(&config).expect("Failed to create wasm engine");

        let component = Component::from_file(&engine, "sandbox.wasm")
            .context("Failed to load sandbox.wasm")?;

        Ok(Self {
            engine,
            component,
            timeout_seconds,
        })
    }

    /// Execute Python code in the sandbox. Returns the result of the
    /// execution as a json serialized string, or an error if
    /// execution fails or timed out.
    pub fn exec(&mut self, code: &str) -> Result<String> {
        let timeout_seconds = self.timeout_seconds;
        let epoch_deadline = timeout_seconds + EPOCH_DEADLINE_BASE;

        // Set up timeout handling
        let timeout_triggered = Arc::new(AtomicBool::new(false));
        {
            let engine_clone = self.engine.clone();
            let timeout_triggered_clone = timeout_triggered.clone();

            thread::spawn(move || {
                thread::sleep(Duration::from_secs(timeout_seconds));
                timeout_triggered_clone.store(true, Ordering::SeqCst);
                for _ in 0..epoch_deadline {
                    engine_clone.increment_epoch();
                }
            });
        }

        // Create a WASI context
        let mut builder = WasiCtxBuilder::new();
        // Enable stdio access by default
        builder.inherit_stdio();

        let wasi_ctx = MyWasi {
            wasi_ctx: builder.build(),
            table: ResourceTable::new(),
        };

        // Create a store with WASI context
        let mut store = Store::new(&self.engine, wasi_ctx);
        store.set_epoch_deadline(epoch_deadline);

        // Set up linker with WASI
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

        // Instantiate the component
        let wasm_sandbox = Sandbox::instantiate(&mut store, &self.component, &linker)?;

        // Execute the code
        let result = wasm_sandbox.call_exec(&mut store, code);
        match result {
            Ok(Ok(val)) => Ok(val),
            Ok(Err(e)) => Err(anyhow!("exec error: {}", e)),
            Err(e) => {
                if timeout_triggered.load(Ordering::SeqCst) {
                    return Err(anyhow!("Execution timed out"));
                }
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_sandbox_default_timeout() {
        // This test will fail if sandbox.wasm doesn't exist, which is expected
        // in a fresh environment. We'll handle this gracefully.
        if !Path::new("sandbox.wasm").exists() {
            return; // Skip test if sandbox.wasm doesn't exist
        }

        let sandbox = PySandbox::new_for_test(None).expect("Failed to create sandbox");
        assert_eq!(sandbox.timeout_seconds, DEFAULT_TIMEOUT_SECONDS);
    }

    #[test]
    fn test_sandbox_custom_timeout() {
        if !Path::new("sandbox.wasm").exists() {
            return;
        }

        let sandbox = PySandbox::new_for_test(Some(10)).expect("Failed to create sandbox");
        assert_eq!(sandbox.timeout_seconds, 10);
    }

    #[test]
    fn test_sandbox_creates_successfully_with_wasm() {
        // Test that sandbox creation succeeds when sandbox.wasm exists
        if !Path::new("sandbox.wasm").exists() {
            return; // Skip test if sandbox.wasm doesn't exist
        }

        let result = PySandbox::new_for_test(None);
        assert!(result.is_ok());
    }
}
