#![no_main]

use libfuzzer_sys::fuzz_target;
#[macro_use]
extern crate libfuzzer_sys;

use template_exercisme::fizz_buzz_fibonacci;

fuzz_target!(|data: u32| {
    fizz_buzz_fibonacci(data);
});
