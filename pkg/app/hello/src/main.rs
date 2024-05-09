#![no_std]
#![no_main]

use lib::*;

extern crate lib;

fn main() -> isize {
    println!("before sleep");
    let start = sys_time();
    println!("{:#?}",start);
    sleep(2000);
    let end = sys_time();
    println!("{:#?}", end);
    println!("Hello, world!!!");
    print!("Please input a line:");
    let stdin1 = stdin();
    let s = stdin1.read_line();
    println!("{}", s);
    
    //loop{}
    233
}

entry!(main);
