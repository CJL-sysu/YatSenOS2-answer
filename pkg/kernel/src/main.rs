#![no_std]
#![no_main]

use ysos::*;
use ysos_kernel as ysos;
use crate::drivers::input;
use crate::interrupt::clock::read_counter;
extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);

    loop {
        print!("> ");
        let input = input::get_line();

        match input.trim() {
            "exit" => break,
            _ => {
                println!("You said: {}", input);
                println!("The counter value is {}", read_counter());
            }
        }
    }

    ysos::shutdown(boot_info);
}