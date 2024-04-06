use core::sync::atomic::{AtomicU16, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(pub u16);

impl ProcessId {
    pub fn new() -> Self {
        // FIXME: Get a unique PID
        // 定义全局计数器
        static COUNTER: AtomicU16 = AtomicU16::new(1);
        // 会原子地递增 COUNTER 的值，并返回递增前的旧值。这样，每次调用 UniqueId::new() 都会获得一个不重复的 UniqueId。
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        ProcessId(id)
    }
}

impl Default for ProcessId {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for ProcessId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ProcessId> for u16 {
    fn from(pid: ProcessId) -> Self {
        pid.0
    }
}
