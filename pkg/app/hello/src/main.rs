#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    println!("Hello, world!!!");
    print!("Please input a line:");
    let stdin1 = stdin();
    let s = stdin1.read_line();
    println!("{}", s);
    
    //loop{}
    233
}

entry!(main);
