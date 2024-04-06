use core::sync::atomic::AtomicU64;

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{memory::gdt, proc::ProcessContext};

use super::consts::*;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
        .set_handler_fn(clock_handler)
        .set_stack_index(gdt::CLOCK_IST_INDEX);
}

pub extern "C" fn clock(mut context: ProcessContext){
    crate::proc::switch(&mut context);
    super::ack();
}
as_handler!(clock);