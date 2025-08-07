/// Defines a sensor 'reader' that will return the current state of the sensor
pub trait SensorReader {
    /// The successful output type
    type Output;
    /// Read the sensor and return the result
    fn read(&self) -> Result<Self::Output, String>;
}


