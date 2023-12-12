contract;

use std::storage::storage_api::{read, write};
use std::context::msg_amount;

abi MyContract {
    #[storage(write)]
    fn initialize_counter(value: u64);
    #[storage(read)]
    fn get_counter() -> u64;
    #[storage(write)]
    fn init_x(value: u64);
    #[storage(read)]
    fn get_x() -> u64;
}

storage {
    value: u64 = 0,
    x: u64 = 0,
}

impl MyContract for Contract {
    #[storage(write)]
    fn initialize_counter(value: u64) {
        storage.value.write(value);
    }
    #[storage(write)]
    fn init_x(value: u64) {
        storage.x.write(value);
    }
    #[storage(read)]
    fn get_x() -> u64 {
        storage.x.read()
    }
    #[storage(read)]
    fn get_counter() -> u64 {
        storage.value.read()
    }
}
