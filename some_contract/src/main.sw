contract;

use std::storage::storage_api::{read, write};
use std::context::msg_amount;

abi MyContract {
    #[storage(write)]
    fn initialize_counter(value: u64);
    #[storage(read)]
    fn get_counter() -> u64;
}

storage {
    value: u64 = 0,
}

impl MyContract for Contract {
    #[storage(write)]
    fn initialize_counter(value: u64) {
        storage.value.write(value);
    }
    #[storage(read)]
    fn get_counter() -> u64 {
        storage.value.read()
    }
}
