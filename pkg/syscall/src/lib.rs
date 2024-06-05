#![no_std]

use num_enum::FromPrimitive;

pub mod macros;

#[repr(usize)]
#[derive(Clone, Debug, FromPrimitive)]
pub enum Syscall {
    Read = 0,
    Write = 1,

    GetPid = 39,
    
    Fork = 58,
    Spawn = 59,
    Exit = 60,
    WaitPid = 61,
    Sem = 66,
    Time = 201,
    ListDir = 114,
    ListApp = 65531,
    Stat = 65532,
    Allocate = 65533,
    Deallocate = 65534,

    #[num_enum(default)]
    Unknown = 65535,
}
