#[cfg(feature = "esp-hal")]
mod esp_hal {
    use esp_hal::{Async, Blocking};

    use crate::esp_hal::{Rs485Uart, UartError};
    use core::ops::Add;
    impl<Dir: embedded_hal::digital::OutputPin> ww_bear::SerialPort for Rs485Uart<'_, Dir, Blocking> {
        type Error = UartError;
        type Instant = esp_hal::time::Instant;

        fn baud_rate(&self) -> Result<u32, Self::Error> {
            Ok(Rs485Uart::baud_rate(self))
        }

        fn set_baud_rate(&mut self, baud_rate: u32) -> Result<(), Self::Error> {
            Rs485Uart::set_baud_rate(self, baud_rate)?;
            Ok(())
        }

        fn discard_input_buffer(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn read(
            &mut self,
            buffer: &mut [u8],
            deadline: &Self::Instant,
        ) -> Result<usize, Self::Error> {
            if deadline < &Self::Instant::now() {
                return Err(Self::Error::Timeout);
            }
            let read = Rs485Uart::read(self, buffer)?;
            Ok(read)
        }

        fn write_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            Rs485Uart::write_all(self, buffer)
        }

        fn make_deadline(&self, timeout: core::time::Duration) -> Self::Instant {
            let micros = timeout.as_micros();
            Self::Instant::now().add(esp_hal::time::Duration::from_micros(micros as u64))
        }

        fn is_timeout_error(error: &Self::Error) -> bool {
            matches!(error, Self::Error::Timeout)
        }
    }

    impl<Dir: embedded_hal::digital::OutputPin> ww_bear::asynchronous::SerialPort
        for Rs485Uart<'_, Dir, Async>
    {
        type Error = UartError;
        type Instant = embassy_time::Instant;

        fn baud_rate(&self) -> Result<u32, Self::Error> {
            Ok(Rs485Uart::baud_rate(self))
        }

        fn set_baud_rate(&mut self, baud_rate: u32) -> Result<(), Self::Error> {
            Rs485Uart::set_baud_rate(self, baud_rate)?;
            Ok(())
        }

        fn discard_input_buffer(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn read(
            &mut self,
            buffer: &mut [u8],
            deadline: &Self::Instant,
        ) -> Result<usize, Self::Error> {
            if deadline < &Self::Instant::now() {
                return Err(Self::Error::Timeout);
            }
            let read = Rs485Uart::read_async(self, buffer, deadline).await?;
            Ok(read)
        }

        async fn write_all(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
            Rs485Uart::write_all_async(self, buffer).await
        }

        fn make_deadline(&self, timeout: core::time::Duration) -> Self::Instant {
            let micros = timeout.as_micros();
            Self::Instant::now().add(embassy_time::Duration::from_micros(micros as u64))
        }

        fn is_timeout_error(error: &Self::Error) -> bool {
            matches!(error, Self::Error::Timeout)
        }
    }
}
