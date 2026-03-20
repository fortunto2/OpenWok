#!/bin/bash
set -e
cd "$(dirname "$0")/../crates/worker"
RUSTC_WRAPPER= worker-build --release
