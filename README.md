# About

Experimental Python WebAssembly sandbox for use with AI agents.

## Run it

First build the python wasm component:

```
uv run build_component.py
```

This produces `sandbox.wasm`, which is embedded into the binary at compile
time via `include_bytes!`. It must exist before running `cargo build` or
`cargo install`; a fresh checkout cannot compile until this step is done.

Run some python code and get back the value of the last expression:

```
cargo run --release - <<'PY'
def fibonacci(n):
    seq = [0, 1]
    while len(seq) < n:
        seq.append(seq[-1] + seq[-2])
    return seq[:n]
fibonacci(10)
PY
```

Install `pybox` locally using `cargo`:

```
cargo install --path .
```

## Sandbox guarantees

Guest code runs in a WASM component with no ambient capabilities:

- No filesystem, environment variables, network, subprocesses, or `ctypes`.
- No access to the host's stdin/stdout/stderr. Guest stdin is empty (reads see
  EOF), and guest stdout/stderr are captured into bounded buffers and returned
  as separate fields (`ExecResult { value, stdout, stderr }`).
- Bounded memory (512 MiB) and captured output (1 MiB per stream).
- Wall-clock timeout via epoch interruption, enforced by a single background
  ticker per engine.
- The `sandbox.wasm` component is embedded in the binary, so no executable code
  is loaded from the working directory.

The CLI writes guest stdout/stderr to the host's stderr and only the result
value to the host's stdout.

## Micro-benchmarks

```
Benchmark 1: pybox "for i in range(10_000_000): pass"
  Time (mean ± σ):      1.396 s ±  0.029 s    [User: 1.359 s, System: 0.027 s]
  Range (min … max):    1.370 s …  1.457 s    10 runs

Benchmark 2: python -c "for i in range(10_000_000): pass"
  Time (mean ± σ):     391.3 ms ± 124.2 ms    [User: 215.6 ms, System: 14.6 ms]
  Range (min … max):   302.9 ms … 732.1 ms    10 runs

Summary
  python -c "for i in range(10_000_000): pass" ran
    3.57 ± 1.13 times faster than pybox "for i in range(10_000_000): pass"
```
