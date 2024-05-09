use core::sync::atomic::AtomicU64;
use crate::guard_access_fn;
use boot::{BootInfo, RuntimeServices, Time};
use chrono::naive::{NaiveDate, NaiveDateTime};
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


once_mutex!(UEFI_SERVICE: UefiRuntime);
pub fn init(boot_info: &'static BootInfo) {
    unsafe {
        init_UEFI_SERVICE(UefiRuntime::new(boot_info));
    }
}
guard_access_fn! {
    pub get_uefi_runtime(UEFI_SERVICE: UefiRuntime)
}
pub struct UefiRuntime {
    runtime_service: &'static RuntimeServices,
}

impl UefiRuntime {
    pub unsafe fn new(boot_info: &'static BootInfo) -> Self {
        Self {
            runtime_service: boot_info.system_table.runtime_services(),
        }
    }

    pub fn get_time(&self) -> Time {
        self.runtime_service.get_time().unwrap()
    }
}
pub fn now() -> NaiveDateTime {
    let time = get_uefi_runtime_for_sure().get_time();
    NaiveDate::from_ymd_opt(time.year() as i32, time.month() as u32, time.day() as u32)
        .unwrap_or_default()
        .and_hms_nano_opt(
            time.hour() as u32,
            time.minute() as u32,
            time.second() as u32,
            time.nanosecond(),
        )
        .unwrap_or_default()
}
