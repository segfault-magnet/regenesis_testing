#!/usr/bin/env bash
num_files="$(find fuel-core | wc -l)"

if [[ $num_files == "1" ]]; then
	echo "Be sure to checkout the 'fuel-core' submodule: git submodule update --init --recursive"
	exit 1
fi

echo "Building fuel-core"
(cd fuel-core && cargo xtask build)
echo "Building forc"
(cd some_contract && forc build)
echo "Running tests"
PATH="./fuel-core/target/debug/:$PATH"
exec cargo test
