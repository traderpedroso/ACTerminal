use sysinfo::{Pid, System};

pub struct SystemMetrics {
    name: String,
    pid: Pid,
    cpu_usage: f32,
    memory_bytes: u64,
}

impl SystemMetrics {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            pid: Pid::from_u32(std::process::id()),
            cpu_usage: 0.0,
            memory_bytes: 0,
        }
    }

    pub fn collect(&mut self, sys: &System) {
        if let Some(process) = sys.process(self.pid) {
            self.cpu_usage = process.cpu_usage();
            self.memory_bytes = process.memory();
        }
    }

    pub fn cpu(&self) -> f64 {
        self.cpu_usage as f64
    }

    pub fn memory(&self) -> u64 {
        self.memory_bytes
    }
}
