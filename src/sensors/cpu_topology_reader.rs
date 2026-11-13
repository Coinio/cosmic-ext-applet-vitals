use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Where the kernel describes each CPU, i.e. each hardware thread.
pub const SYSFS_CPU_ROOT: &str = "/sys/devices/system/cpu";

/// Which physical core each CPU, i.e. each hardware thread, belongs to.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CpuTopology {
    /// The physical core of each CPU by its kernel number. Cores are numbered from 0, in the order of their first CPU.
    core_of_cpu: BTreeMap<usize, usize>,
}

impl CpuTopology {
    /// Read the topology of the online CPUs. Offline CPUs aren't described, so they're missing from it.
    pub fn read(root: &Path) -> Self {
        let Ok(entries) = fs::read_dir(root) else {
            return CpuTopology::default();
        };

        let mut siblings_of_cpu: BTreeMap<usize, String> = BTreeMap::new();
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let Some(cpu) = file_name
                .to_str()
                .and_then(|name| name.strip_prefix("cpu"))
                .and_then(|number| number.parse::<usize>().ok())
            else {
                continue;
            };

            let topology = entry.path().join("topology");
            // `core_cpus_list` replaced `thread_siblings_list` in Linux 5.6, which older kernels only have.
            let siblings = fs::read_to_string(topology.join("core_cpus_list"))
                .or_else(|_| fs::read_to_string(topology.join("thread_siblings_list")));

            if let Ok(siblings) = siblings {
                siblings_of_cpu.insert(cpu, siblings.trim().to_string());
            }
        }

        Self::from_siblings(siblings_of_cpu)
    }

    /// CPUs that list the same siblings are the hardware threads of one core.
    fn from_siblings(siblings_of_cpu: BTreeMap<usize, String>) -> Self {
        let mut core_of_siblings: BTreeMap<String, usize> = BTreeMap::new();
        let mut core_of_cpu = BTreeMap::new();

        for (cpu, siblings) in siblings_of_cpu {
            let next_core = core_of_siblings.len();
            let core = *core_of_siblings.entry(siblings).or_insert(next_core);
            core_of_cpu.insert(cpu, core);
        }

        CpuTopology { core_of_cpu }
    }

    /// The physical core the CPU belongs to, if it's known.
    pub fn core_of(&self, cpu: usize) -> Option<usize> {
        self.core_of_cpu.get(&cpu).copied()
    }

    /// The number of physical cores.
    pub fn core_count(&self) -> usize {
        self.core_of_cpu.values().collect::<std::collections::BTreeSet<_>>().len()
    }

    /// The number of hardware threads.
    pub fn thread_count(&self) -> usize {
        self.core_of_cpu.len()
    }

    #[cfg(test)]
    pub fn from_cores(core_of_cpu: impl IntoIterator<Item = (usize, usize)>) -> Self {
        CpuTopology {
            core_of_cpu: core_of_cpu.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_cpu(root: &Path, cpu: usize, file: &str, siblings: &str) {
        let topology = root.join(format!("cpu{cpu}")).join("topology");
        fs::create_dir_all(&topology).unwrap();
        fs::write(topology.join(file), format!("{siblings}\n")).unwrap();
    }

    #[test]
    fn groups_the_threads_of_each_core() {
        let root = tempfile::tempdir().unwrap();
        // Two cores with two threads each, numbered the way AMD and Intel desktop CPUs are.
        write_cpu(root.path(), 0, "core_cpus_list", "0,2");
        write_cpu(root.path(), 1, "core_cpus_list", "1,3");
        write_cpu(root.path(), 2, "core_cpus_list", "0,2");
        write_cpu(root.path(), 3, "core_cpus_list", "1,3");
        // Not CPUs, so they're skipped.
        fs::create_dir_all(root.path().join("cpufreq")).unwrap();
        fs::create_dir_all(root.path().join("cpuidle")).unwrap();

        let topology = CpuTopology::read(root.path());

        assert_eq!(topology.core_of(0), Some(0));
        assert_eq!(topology.core_of(1), Some(1));
        assert_eq!(topology.core_of(2), Some(0));
        assert_eq!(topology.core_of(3), Some(1));
        assert_eq!(topology.core_of(4), None);
        assert_eq!(topology.core_count(), 2);
        assert_eq!(topology.thread_count(), 4);
    }

    #[test]
    fn falls_back_to_thread_siblings_on_older_kernels() {
        let root = tempfile::tempdir().unwrap();
        write_cpu(root.path(), 0, "thread_siblings_list", "0-1");
        write_cpu(root.path(), 1, "thread_siblings_list", "0-1");
        write_cpu(root.path(), 2, "thread_siblings_list", "2");

        let topology = CpuTopology::read(root.path());

        assert_eq!(topology.core_of(0), Some(0));
        assert_eq!(topology.core_of(1), Some(0));
        assert_eq!(topology.core_of(2), Some(1));
    }

    #[test]
    fn a_missing_root_has_no_cores() {
        let topology = CpuTopology::read(Path::new("/nonexistent/cpu/root"));
        assert_eq!(topology, CpuTopology::default());
    }
}
