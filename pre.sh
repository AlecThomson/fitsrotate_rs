#!/usr/bin/env bash

# Install deps
apk add --no-cache cfitsio
apk add --no-cache cfitsio-dev

# Update rust
rustup update

# Check rust version
rustc --version