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
    // unsafe{
    //     // let a:*mut u8 = 0x1111111111111 as *mut u8;
    //     // *a = 99;
    //     let a:*mut u8 = 0x1000000000 as *mut u8;
    //     let b:u8 = *a + 1;
    //     println!("{b}");
    // }
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