#!/usr/bin/env bash

PATH="./fuel-core/target/debug/:$PATH"
exec cargo test
