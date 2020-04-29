#!/bin/bash

SCENARIO_NAME='Simple Library'

BOLD='\033[1m'
RESET='\033[0m'

function print_message {
    echo
    echo -e "${BOLD}$(echo $@)${RESET}"
    echo
}

# signal error
function signal_error {
    print_message "######## Scenario failed: ${SCENARIO_NAME} ########"
    exit 1
}

# catch any errors
trap 'signal_error' ERR

# start the scenario
print_message "######## Starting scenario: ${SCENARIO_NAME} ########"

print_message "######## Creating a library ########"

# create a new library
cargo new --registry "${REGISTRY_NAME}" --lib simple-lib
cd simple-lib

# implement some simple functionality
cat > src/lib.rs << EOF
use std::ops::{Add, Mul};
pub fn amazing_add<T: Add<T>>(a: T, b: T) -> T::Output { a + b }
pub fn amazing_multiply<T: Mul<T>>(a: T, b: T) -> T::Output { a * b }
EOF

print_message "######## Publishing the library ########"

# publish it to the registry
cargo publish --registry "${REGISTRY_NAME}" --allow-dirty

# go back to parent directory
cd ..

print_message "######## Creating a binary (that depends on the library) ########"

# create a new binary
cargo new --registry "${REGISTRY_NAME}" --bin simple-bin
cd simple-bin

# implement some simple functionality
cat > src/main.rs << EOF
use simple_lib::{amazing_add, amazing_multiply};
fn main() {
    println!("5 + 2 = {}", amazing_add(5, 2));
    println!("5 * 2 = {}", amazing_multiply(5, 2));
}
EOF

# declare dependancy on `simple-lib`
echo "simple-lib = { registry = \"${REGISTRY_NAME}\", version = \"0.1.0\" }" >> Cargo.toml

print_message "######## Building and running the binary ########"

# attempt to build and run this binary (which should pull and build the library successfully)
cargo run

# everything worked, test succeeded
print_message "######## Scenario succeeded: ${SCENARIO_NAME} ########"
