#!/usr/bin/env python3
"""Build script to generate sandbox.wasm from guest.py using componentize-py."""

import subprocess
import sys

def main():
    print("Building sandbox.wasm from guest.py...")
    cmd = [
        "componentize-py",
        "-d", "sandbox.wit",
        "componentize",
        "--stub-wasi",
        "guest",
        "-o", "sandbox.wasm"
    ]

    result = subprocess.run(cmd, check=True)
    print("Successfully built sandbox.wasm")
    return 0

if __name__ == "__main__":
    sys.exit(main())