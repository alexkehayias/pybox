use anyhow::{anyhow, Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use wasmtime::{Config, Engine, Store};
use wasmtime::component::{Component, Linker};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView};

const TIMEOUT_SECONDS: u64 = 40;
const EPOCH_DEADLINE: u64 = TIMEOUT_SECONDS + 1;


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

// ---------- Bindings generated from sandbox.wit ----------
wasmtime::component::bindgen!({
    path: "sandbox.wit",
    world: "sandbox",
});

// ---------- Helper to build engine ----------
fn make_engine() -> Engine {
    let mut cfg = Config::new();
    cfg.epoch_interruption(true); // for timeout support
    Engine::new(&cfg).expect("engine creation")
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("usage: pybox2 [<statement>...] <expression>");
        std::process::exit(-1);
    }

    // Read code from stdin if '-' is passed
    let code = if args.len() == 1 && args[0] == "-" {
        use std::io::{self, Read};
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        input.trim().to_string()
    } else {
        args.last().unwrap().clone()
    };

    // Configure Wasmtime engine with epoch interruption for timeout support
    let engine = make_engine();

    // Set up timeout handling
    let timeout_triggered = Arc::new(AtomicBool::new(false));
    {
        let engine_clone = engine.clone();
        let timeout_triggered_clone = timeout_triggered.clone();

        thread::spawn(move || {
            thread::sleep(Duration::from_secs(TIMEOUT_SECONDS));
            timeout_triggered_clone.store(true, Ordering::SeqCst);
            for _ in 0..EPOCH_DEADLINE {
                engine_clone.increment_epoch();
            }
        });
    }

    // Create a WASI context
    let mut builder = WasiCtxBuilder::new();
    builder.inherit_stdio();

    let wasi_ctx = MyWasi {
        wasi_ctx: builder.build(),
        table: ResourceTable::new(),
    };

    // Create a store with WASI context
    let mut store = Store::new(&engine, wasi_ctx);
    store.set_epoch_deadline(EPOCH_DEADLINE);

    // Load the component
    let component = Component::from_file(&engine, "sandbox.wasm")
        .context("Failed to load sandbox.wasm")?;

    // Set up linker with WASI
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    // Instantiate the component
    let sandbox = Sandbox::instantiate(&mut store, &component, &linker)?;

    // Execute the code
    let result = sandbox.call_exec(&mut store, &code);
    match result {
        Ok(Ok(val)) => {
            println!("{}", val);
        }
        Ok(Err(e)) => {
            eprintln!("Error: {}", e);
            std::process::exit(-1);
        }
        Err(e) => {
            if timeout_triggered.load(Ordering::SeqCst) {
                eprintln!("Error: timeout");
            }
            return Err(e);
        }
    }

    // Check if timeout was triggered
    if timeout_triggered.load(Ordering::SeqCst) {
        eprintln!("Error: timeout");
        return Err(anyhow!("Execution timed out"));
    }

    Ok(())
}
