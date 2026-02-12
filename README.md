# About

Experimental Python WebAssembly sandbox for use with AI agents.

## Run it

First build the python wasm component:

```
uv run build_component.py
```

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
