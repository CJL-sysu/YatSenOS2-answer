mod apic;
mod consts;
pub mod clock;
mod serial;
mod exceptions;
pub mod syscall;

use apic::*;
use x86_64::{registers::debug, structures::idt::InterruptDescriptorTable};
use crate::{memory::physical_to_virtual, proc::{self, get_process_manager, sync::SemaphoreResult, ProcessContext}};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            exceptions::register_idt(&mut idt);
            clock::register_idt(&mut idt);
            serial::register_idt(&mut idt);
            syscall::register_idt(&mut idt);    
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

pub fn new_sem(key:u32, value:usize)-> usize{
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().current().write().new_sem(key, value) as usize
    })
}

pub fn remove_sem(key:u32)-> usize{
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().current().write().remove_sem(key) as usize
    })
}

pub fn sem_signal(key:u32, context: &mut ProcessContext){
    //debug!("signal is called");
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        match manager.current().write().sem_signal(key) {
            SemaphoreResult::Ok => {
                context.set_rax(0);
            },
            SemaphoreResult::NotExist => {
                context.set_rax(1);
            },
            SemaphoreResult::WakeUp(pid) => {
                manager.wake_up(pid);
            }
            SemaphoreResult::Block(pid)=>{
                manager.block(pid);
                error!("pid = {} is blocked by signal, which is a fatal bug. Plz report this bug to CJL", pid);
            }
        }
    })
}

pub fn sem_wait(key:u32, context: &mut ProcessContext){
    //debug!("wait is called");
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let pid = manager.current().pid();
        let ret = manager.current().write().sem_wait(key, pid);
        match ret {
            SemaphoreResult::Ok => {
                //debug!("is ok");
                context.set_rax(0);
            },
            SemaphoreResult::NotExist => {
                context.set_rax(1);
            },
            SemaphoreResult::WakeUp(pid) => {
                manager.wake_up(pid);
                error!("pid = {} is waked by wait, which is a fatal bug. Plz report this bug to CJL", pid);
            }
            SemaphoreResult::Block(pid)=>{
                //debug!("is blocked");
                manager.save_current(context);
                manager.block(pid);
                manager.switch_next(context);
            }
        }
    })
}