use core::fmt;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};
/// A port-mapped UART 16550 serial interface.
//Fix: struct
pub struct SerialPort{
    data: Port<u8>,
    interrupt_enable: PortWriteOnly<u8>,
    interrupt_identification: PortWriteOnly<u8>,
    line_control: PortWriteOnly<u8>,
    modem_control: PortWriteOnly<u8>,
    line_status: PortReadOnly<u8>,
}

impl SerialPort {
    pub const fn new(port: u16) -> Self {
        //FIX: init
        Self{
            data: Port::new(port + 0),
            interrupt_enable: PortWriteOnly::new(port + 1),
            interrupt_identification: PortWriteOnly::new(port + 2),
            line_control: PortWriteOnly::new(port + 3),
            modem_control: PortWriteOnly::new(port + 4),
            line_status: PortReadOnly::new(port + 5),
        }
    }

    /// Initializes the serial port.
    pub fn init(&mut self) {//这里原来是不可变借用，改成了可变借用
        // FIXME: Initialize the serial port
        unsafe{
            // Disable all interrupts
            self.interrupt_enable.write(0x00);
            // Enable DLAB (set baud rate divisor)
            self.line_control.write(0x80);
            // Set divisor to 3 (lo byte) 38400 baud
            self.data.write(0x03);
            // Set divisor to 3 (hi byte) 38400 baud
            self.interrupt_enable.write(0x00);
            // 8 bits, no parity, one stop bit
            self.line_control.write(0x03);
            // Enable FIFO, clear them, with 14-byte threshold
            self.interrupt_identification.write(0xc7);
            // IRQs enabled, RTS/DSR set
            self.modem_control.write(0x0B);
            // Set in loopback mode, test the serial chip
            self.modem_control.write(0x1e);
            // Test serial chip (send byte 0xAE and check if serial returns same byte)
            self.data.write(0xae);
            // Check if serial is faulty (i.e: not same byte as sent)
            if self.data.read()!= 0xae{
                panic!("serial is faulty");
            }
            // If serial is not faulty set it in normal operation mode
            // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
            self.modem_control.write(0x0f);
            // Enable interrupts
            self.interrupt_enable.write(0x01);
        }
    }

    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        // FIXME: Send a byte on the serial port
        unsafe{
            while self.line_status.read() & 0x20 ==0{}
            self.data.write(data);
        }
    }

    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {
        // FIXME: Receive a byte on the serial port no wait
        unsafe{
            if self.line_status.read() & 1 !=0{
                Some(self.data.read())
            }else{
                None
            }
            
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}
