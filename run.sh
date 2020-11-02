#!/bin/bash
RUST_BACKTRACE=1 RUST_LOG="ninja_force=debug" cargo +nightly run --release --features metal
