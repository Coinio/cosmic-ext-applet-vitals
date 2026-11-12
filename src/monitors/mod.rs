use std::time::Duration;

pub mod cpu_monitor;
pub mod memory_monitor;
pub mod network_monitor;
pub mod disk_monitor;
pub mod storage_monitor;
pub mod cpu_sensors_monitor;
pub mod gpu_monitor;

/// `total` spread over `elapsed`, per second. Polls aren't always the update interval apart, so the data rates are
/// worked out from the time that actually passed. 0 when no time has passed, e.g. at the first poll.
fn per_second(total: u64, elapsed: Duration) -> u64 {
    let seconds = elapsed.as_secs_f64();
    if seconds <= 0.0 {
        return 0;
    }
    (total as f64 / seconds).round() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_second_divides_by_the_elapsed_time() {
        assert_eq!(per_second(3000, Duration::from_secs(2)), 1500);
        assert_eq!(per_second(3000, Duration::from_millis(500)), 6000);
        assert_eq!(per_second(3000, Duration::ZERO), 0);
    }
}
