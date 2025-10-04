use crate::configuration::app_configuration::AppConfiguration;

pub mod disk;
pub mod network;
pub mod cpu;
pub mod memory;

fn format_bytes_per_second(bytes_per_sec: u64, app_config: &AppConfiguration) -> String {
    if app_config.general.use_iec_units {
        let mib_per_second = bytes_per_sec as f64 / 1024.0 / 1024.0;
        if mib_per_second > 999.9 {
            let gib_per_second = bytes_per_sec as f64 / 1024.0 / 1024.0 / 1024.0;
            format!("{:.1}GiB/s", gib_per_second)
        } else if mib_per_second > 99.9 {
            format!("{:.0}MiB/s", mib_per_second.round())
        } else {
            format!("{:.1}MiB/s", mib_per_second)
        }
    } else {
        let mb_per_second = bytes_per_sec as f64 / 1_000_000.0;
        if mb_per_second > 999.9 {
            let gb_per_second = bytes_per_sec as f64 / 1_000_000_000.0;
            format!("{:.1}GB/s", gb_per_second)
        } else if mb_per_second > 99.9 {
            format!("{:.0}MB/s", mb_per_second.round())
        } else {
            format!("{:.1}MB/s", mb_per_second)
        }
    }
}