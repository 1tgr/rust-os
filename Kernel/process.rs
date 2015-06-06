use super::arch::process::ArchProcess;

pub struct Process {
    arch: ArchProcess
}

impl Process {
    pub fn new() -> Process {
        Process {
            arch: ArchProcess::new()
        }
    }

    pub fn kernel() -> Process {
        Process {
            arch: ArchProcess::kernel()
        }
    }
}
