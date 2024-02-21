#!/usr/bin/env bash

# Install deps
sudo apt-get update -y && sudo apt-get install libcfitsio-dev -y

# Update rust
rustup update

# Check rust version
rustc --version