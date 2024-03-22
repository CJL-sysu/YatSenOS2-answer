mod apic;
mod consts;
pub mod clock;
mod serial;
mod exceptions;

use apic::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use crate::memory::physical_to_virtual;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            exceptions::register_idt(&mut idt);
            clock::register_idt(&mut idt);
            serial::register_idt(&mut idt);
        }
        idt
    };
}

/// init interrupts system
pub fn init() {
    IDT.load();

    // FIXME: check and init APIC
    match apic::XApic::support() {
        true => info!("XAPIC supported"),
        false => error!("XAPIC not supported"),
    }
    let mut xapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    xapic.cpu_init();

    // FIXME: enable serial irq with IO APIC (use enable_irq)
    enable_irq(consts::Irq::Serial0 as u8 , 0);
    //enable_irq(consts::Irq::Keyboard as u8 + consts::Interrupts::IrqBase as u8, 0);
    //enable_irq(consts::Irq::Serial1 as u8 + consts::Interrupts::IrqBase as u8, 0);
    info!("Interrupts Initialized.");
}

#[inline(always)]
pub fn enable_irq(irq: u8, cpuid: u8) {
    let mut ioapic = unsafe { IoApic::new(physical_to_virtual(IOAPIC_ADDR)) };
    ioapic.enable(irq, cpuid);
}

#[inline(always)]
pub fn ack() {
    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
