/// Defines a sensor 'reader' that will return the current state of the sensor
pub trait SensorReader {
    /// The successful output type
    type Output;
    /// Read what the panel shows, and return the result
    fn read(&self) -> Result<Self::Output, String>;
    /// Read what the panel shows, and what only the detail popup shows, and return the result. The popup's readings
    /// can cost more, e.g. a sysfs file per device, so they're only read while it's wanted.
    fn read_detailed(&self) -> Result<Self::Output, String> {
        self.read()
    }
}
