use crate::sensors::cpu_topology_reader::CpuTopology;
use crate::sensors::proc_file;
use std::fs;
use std::path::{Path, PathBuf};

/// Where the kernel describes the CPU, including its model name.
pub const PROC_CPUINFO_FILE: &str = "/proc/cpuinfo";

/// What doesn't change about the CPU: its model, how many cores and threads it has, and its highest clock speed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CpuInfo {
    /// The model name, e.g. "AMD Ryzen 7 7700 8-Core Processor". Only x86 CPUs, and a few others, report one.
    pub model: Option<String>,
    /// The physical cores, or 0 when the topology couldn't be read.
    pub cores: usize,
    /// The hardware threads, or 0 when the topology couldn't be read.
    pub threads: usize,
    /// The highest clock speed of any core, with boost, in kHz, when the CPU's frequency driver reports it.
    pub max_frequency_khz: Option<u64>,
}

impl CpuInfo {
    /// Read the model from `cpuinfo_file`, and the rest from the online CPUs in `sysfs_cpu_root`.
    pub fn read(cpuinfo_file: &Path, sysfs_cpu_root: &Path) -> Self {
        let topology = CpuTopology::read(sysfs_cpu_root);
        let model = fs::read_to_string(cpuinfo_file).ok().and_then(|contents| parse_model(&contents));
        let max_frequency_khz = cpufreq_files(sysfs_cpu_root, "cpuinfo_max_freq")
            .iter()
            .filter_map(proc_file::read_value::<u64>)
            .filter(|khz| *khz > 0)
            .max();

        CpuInfo {
            model,
            cores: topology.core_count(),
            threads: topology.thread_count(),
            max_frequency_khz,
        }
    }
}

/// The first CPU's model name. Every CPU lists it, and they're the same on all but the rarest multi-socket machines.
/// Some drivers pad it with spaces, so runs of them are collapsed.
fn parse_model(cpuinfo: &str) -> Option<String> {
    cpuinfo
        .lines()
        .filter_map(|line| line.split_once(':'))
        .find(|(key, _)| key.trim() == "model name")
        .map(|(_, value)| value.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|model| !model.is_empty())
}

/// The `file` in each CPU's `cpufreq` directory, for the CPUs that have one.
fn cpufreq_files(sysfs_cpu_root: &Path, file: &str) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(sysfs_cpu_root) else {
        return Vec::new();
    };

    let mut files: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_prefix("cpu"))
                .is_some_and(|number| number.parse::<usize>().is_ok())
        })
        .map(|entry| entry.path().join("cpufreq").join(file))
        .filter(|path| path.exists())
        .collect();
    files.sort();
    files
}

/// Reads the current clock speed of the CPU, averaged over its threads.
pub struct CpuFrequencyReader {
    /// Each thread's current frequency file, found once. A thread that goes offline can't be read, so it's skipped.
    files: Vec<PathBuf>,
}

impl CpuFrequencyReader {
    pub fn new(sysfs_cpu_root: &Path) -> Self {
        Self {
            files: cpufreq_files(sysfs_cpu_root, "scaling_cur_freq"),
        }
    }

    /// The average current frequency in kHz, or `None` when the CPU's frequency driver doesn't report it, e.g. in
    /// many virtual machines.
    pub fn read_average_khz(&self) -> Option<u64> {
        let frequencies: Vec<u64> = self
            .files
            .iter()
            .filter_map(proc_file::read_value::<u64>)
            .filter(|khz| *khz > 0)
            .collect();

        (!frequencies.is_empty()).then(|| frequencies.iter().sum::<u64>() / frequencies.len() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A CPU in a fake `/sys/devices/system/cpu`, in its own core, with its frequency files.
    fn write_cpu(root: &Path, cpu: usize, current_khz: Option<u64>, max_khz: Option<u64>) {
        let dir = root.join(format!("cpu{cpu}"));
        fs::create_dir_all(dir.join("topology")).unwrap();
        fs::write(dir.join("topology").join("core_cpus_list"), format!("{cpu}\n")).unwrap();
        fs::create_dir_all(dir.join("cpufreq")).unwrap();
        if let Some(khz) = current_khz {
            fs::write(dir.join("cpufreq").join("scaling_cur_freq"), format!("{khz}\n")).unwrap();
        }
        if let Some(khz) = max_khz {
            fs::write(dir.join("cpufreq").join("cpuinfo_max_freq"), format!("{khz}\n")).unwrap();
        }
    }

    #[test]
    fn parses_the_model_name() {
        let cpuinfo = "processor\t: 0\nvendor_id\t: AuthenticAMD\nmodel\t\t: 97\n\
                       model name\t: AMD Ryzen 7 7700 8-Core Processor\n\nprocessor\t: 1\n\
                       model name\t: AMD Ryzen 7 7700 8-Core Processor\n";
        assert_eq!(parse_model(cpuinfo).as_deref(), Some("AMD Ryzen 7 7700 8-Core Processor"));

        let padded = "model name\t: Intel(R) Core(TM) i7-4790K CPU @ 4.00GHz   \n";
        assert_eq!(parse_model(padded).as_deref(), Some("Intel(R) Core(TM) i7-4790K CPU @ 4.00GHz"));

        // ARM CPUs don't have a model name, only part numbers.
        let arm = "processor\t: 0\nBogoMIPS\t: 108.00\nCPU implementer\t: 0x41\nCPU part\t: 0xd08\n";
        assert_eq!(parse_model(arm), None);
    }

    #[test]
    fn reads_the_cores_threads_and_highest_clock() {
        let root = tempfile::tempdir().unwrap();
        let cpu_root = root.path().join("cpu");
        // A hybrid CPU, whose performance cores clock higher than its efficiency cores.
        write_cpu(&cpu_root, 0, None, Some(5_000_000));
        write_cpu(&cpu_root, 1, None, Some(3_800_000));
        write_cpu(&cpu_root, 2, None, Some(5_400_000));
        fs::create_dir_all(cpu_root.join("cpufreq")).unwrap();
        let cpuinfo = root.path().join("cpuinfo");
        fs::write(&cpuinfo, "model name\t: Some CPU\n").unwrap();

        assert_eq!(
            CpuInfo::read(&cpuinfo, &cpu_root),
            CpuInfo {
                model: Some("Some CPU".to_string()),
                cores: 3,
                threads: 3,
                max_frequency_khz: Some(5_400_000),
            }
        );
    }

    #[test]
    fn missing_files_leave_the_info_unknown() {
        let info = CpuInfo::read(Path::new("/nonexistent/cpuinfo"), Path::new("/nonexistent/cpu"));
        assert_eq!(info, CpuInfo::default());
    }

    #[test]
    fn averages_the_current_frequency_of_the_threads() {
        let root = tempfile::tempdir().unwrap();
        write_cpu(root.path(), 0, Some(4_000_000), None);
        write_cpu(root.path(), 1, Some(2_000_000), None);
        // No frequency driver for this one.
        write_cpu(root.path(), 2, None, None);

        let reader = CpuFrequencyReader::new(root.path());
        assert_eq!(reader.read_average_khz(), Some(3_000_000));

        fs::write(root.path().join("cpu1").join("cpufreq").join("scaling_cur_freq"), "1000000\n").unwrap();
        assert_eq!(reader.read_average_khz(), Some(2_500_000));
    }

    /// Reads the CPU of the machine the tests run on. Run with `cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn reads_the_cpu_of_this_machine() {
        let root = Path::new(crate::sensors::cpu_topology_reader::SYSFS_CPU_ROOT);
        let info = CpuInfo::read(Path::new(PROC_CPUINFO_FILE), root);
        let frequency = CpuFrequencyReader::new(root).read_average_khz();
        println!("{info:?}, current: {frequency:?} kHz");
        assert!(info.cores > 0 && info.threads >= info.cores);
    }

    #[test]
    fn no_frequency_driver_has_no_frequency() {
        let reader = CpuFrequencyReader::new(Path::new("/nonexistent/cpu"));
        assert_eq!(reader.read_average_khz(), None);
    }
}
