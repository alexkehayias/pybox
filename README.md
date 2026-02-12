# About

Experimental Python WebAssembly sandbox.

## Run it

```
uv run build_component.py
cargo run --release - <<'PY'
def fibonacci(n):
    seq = [0, 1]
    while len(seq) < n:
        seq.append(seq[-1] + seq[-2])
    return seq[:n]
fibonacci(10)
PY
```
