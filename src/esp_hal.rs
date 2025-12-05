use esp_hal::uart::{ConfigError, RxError, TxError, Uart};
use esp_hal::{Async, Blocking, DriverMode};

#[derive(Debug, derive_more::Display, derive_more::From, derive_more::Error)]
pub enum UartError {
    TxError(TxError),
    RxError(RxError),
    DirPinError,
    Timeout,
    SetBaudError(ConfigError),
}
impl From<embassy_time::TimeoutError> for UartError {
    fn from(_: embassy_time::TimeoutError) -> Self {
        UartError::Timeout
    }
}

pub struct Rs485Uart<'a, Dir, Mode: DriverMode> {
    uart: Uart<'a, Mode>,
    dir: Dir,
    baud_rate: u32,
}

impl<'a, Dir: embedded_hal::digital::OutputPin, Mode: DriverMode> Rs485Uart<'a, Dir, Mode> {
    pub fn new(uart: Uart<'a, Mode>, dir: Dir, baud_rate: u32) -> Result<Self, ConfigError> {
        let mut s = Self {
            uart,
            dir,
            baud_rate,
        };
        s.set_baud_rate(baud_rate)?;
        Ok(s)
    }

    pub fn baud_rate(&self) -> u32 {
        self.baud_rate
    }

    pub fn set_baud_rate(&mut self, baud_rate: u32) -> Result<(), ConfigError> {
        self.uart
            .apply_config(&esp_hal::uart::Config::default().with_baudrate(baud_rate))?;
        self.baud_rate = baud_rate;
        Ok(())
    }
}

impl<'a, Dir: embedded_hal::digital::OutputPin> Rs485Uart<'a, Dir, Blocking> {
    pub fn into_async(self) -> Rs485Uart<'a, Dir, Async> {
        let Self {
            uart,
            dir,
            baud_rate,
        } = self;

        let uart = uart.into_async();

        Rs485Uart {
            uart,
            dir,
            baud_rate,
        }
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, UartError> {
        let read = self.uart.read_buffered(buffer)?;
        #[cfg(feature = "defmt")]
        if read > 0 {
            defmt::trace!("read {} bytes", read);
            defmt::trace!("read {:X}", &buffer[..read]);
        }
        Ok(read)
    }

    pub fn write_all(&mut self, mut buffer: &[u8]) -> Result<(), UartError> {
        self.dir.set_high().map_err(|_| UartError::DirPinError)?;
        while !buffer.is_empty() {
            match self.uart.write(buffer)? {
                0 => panic!("write() returned Ok(0)"),
                n => {
                    #[cfg(feature = "defmt")]
                    {
                        defmt::trace!("wrote {} bytes", buffer.len());
                        defmt::trace!("wrote {:X}", buffer);
                    }
                    buffer = &buffer[n..];
                }
            }
        }
        self.uart.flush()?;
        self.dir.set_low().map_err(|_| UartError::DirPinError)?;
        Ok(())
    }
}

impl<'a, Dir: embedded_hal::digital::OutputPin> Rs485Uart<'a, Dir, Async> {
    pub fn into_blocking(self) -> Rs485Uart<'a, Dir, Blocking> {
        let Self {
            uart,
            dir,
            baud_rate,
        } = self;

        let uart = uart.into_blocking();

        Rs485Uart {
            uart,
            dir,
            baud_rate,
        }
    }

    pub async fn read_async(
        &mut self,
        buffer: &mut [u8],
        deadline: &embassy_time::Instant,
    ) -> Result<usize, UartError> {
        use embassy_time::WithTimeout;
        let read = self
            .uart
            .read_async(buffer)
            .with_deadline(*deadline)
            .await??;
        #[cfg(feature = "defmt")]
        if read > 0 {
            defmt::trace!("read {} bytes", read);
            defmt::trace!("read {:X}", &buffer[..read]);
        }
        Ok(read)
    }

    pub async fn write_all_async(&mut self, mut buffer: &[u8]) -> Result<(), UartError> {
        self.dir.set_high().map_err(|_| UartError::DirPinError)?;
        while !buffer.is_empty() {
            match self.uart.write(buffer)? {
                0 => panic!("write() returned Ok(0)"),
                n => buffer = &buffer[n..],
            }
        }
        self.uart.flush()?;
        self.dir.set_low().map_err(|_| UartError::DirPinError)?;
        #[cfg(feature = "defmt")]
        {
            defmt::trace!("wrote {} bytes", buffer.len());
            defmt::trace!("wrote {:X}", buffer);
        }
        Ok(())
    }
}
