use core::str::from_utf8;
use alloc::vec::Vec;
use pc_keyboard::DecodedKey;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{drivers::input::push_key, serial::get_serial_for_sure};

use super::consts::*;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Serial0 as u8]
        .set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    //info!("at serial handler");
    receive();
    super::ack();
}
const INPUT_BUFFER_SIZE: usize = 4;
/// Receive character from uart 16550
/// Should be called on every interrupt
fn receive() {
    //info!("at receive");
    // FIXME: receive character from uart 16550, put it into INPUT_BUFFER
    let mut input_buffer:Vec<u8> = Vec::with_capacity(INPUT_BUFFER_SIZE);
    loop{
        //info!("at receive");
        let mut serial = get_serial_for_sure();
        let rec = serial.receive();
        drop(serial);
        // println!("{:?}", rec);
        // break;

        match rec{
            Some(c) => {
                // info!("at some");
                // info!("{}", c as char);
                input_buffer.push(c);
                if let Ok(s) = from_utf8(&input_buffer){
                    let ch = s.chars().next().unwrap();
                    push_key(DecodedKey::Unicode(ch));
                    input_buffer.clear();
                }
            },
            _ => {
                // info!("failed to receive");
                //println!("failed to receive");
                break;
            }
        }
    }
}