#![allow(unused_imports)]

use embedded_hal::spi;

use crate::{Error, Ready, Sleeping, DW3000};

/// This enum represents the different SleepStates the device can be in
///
/// While the IC is in the SLEEP state, the external system should avoid
/// applying power to GPIO, SPICLK or SPIMISO pins as this will cause an
/// increase in the leakage current.
/// While device is in the SLEEP state the SPI communication is not possible.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SleepState {
    /// Normal sleep
    /// Parameter 0 is written to SLEEP_TIM
    /// Unit is not fixed, as the clock speed of the LF RC is changeable
    Sleep(u16),
    /// Deep sleep, wake up only with WAKE pin or CS pin
    DeepSleep,
}

impl<SPI> DW3000<SPI, Sleeping>
where
    SPI: spi::SpiDevice<u8>,
{
    /// Configures the radio after sleep
    /// This function needs to be called, after the DW3000 has been awaked in order
    /// to reconfigure it
    pub fn finish_wakeup(mut self) -> Result<DW3000<SPI, Ready>, Error<SPI>> {
        // Let's check that we're actually awake now
        if self.ll.dev_id().read()?.ridtag() != 0xDECA {
            // Oh dear... We have not woken up!
            return Err(Error::StillAsleep);
        }

        // Readout `LDO_TUNE` from OTP to make sure it is none zero:
        // Only address 0x4 is of interest, as it is what
        // LDO_KICK copies to the configuration
        self.ll.otp_cfg().modify(|_, w| w.otp_man(1))?;
        self.ll.otp_addr().modify(|_, w| w.otp_addr(0x04))?;
        self.ll.otp_cfg().modify(|_, w| w.otp_read(1))?;
        let ldo_low = self.ll.otp_rdata().read()?.value();

        if ldo_low != 0 {
            self.ll.otp_cfg().modify(|_, w| w.ldo_kick(1))?;
        }

        Ok(DW3000 {
            ll: self.ll,
            seq: self.seq,
            state: Ready,
        })
    }

    /*
    /// Wakes the radio up.
    pub fn wake_up<DELAY: embedded_hal::blocking::delay::DelayUs<u16>>(
        mut self,
        delay: &mut DELAY,
    ) -> Result<DW3000<SPI, Ready>, Error<SPI>> {
        // Wake up using the spi
        self.ll.assert_cs_low().map_err(|e| Error::Spi(e))?;
        delay.delay_us(850 * 2);
        self.ll.assert_cs_high().map_err(|e| Error::Spi(e))?;

        // Now we must wait 4 ms so all the clocks start running.
        delay.delay_us(4000 * 2);

        // Let's check that we're actually awake now
        if self.ll.dev_id().read()?.ridtag() != 0xDECA {
            // Oh dear... We have not woken up!
            return Err(Error::StillAsleep);
        }

        // Reset the wakeupstatus
        self.ll.sys_status().write(|w| w.slp2init(1).cplock(1))?;

        // Restore the tx antenna delay
        let delay = self.state.tx_antenna_delay;
        self.ll.tx_antd().write(|w| w.value(delay.value() as u16))?;

        // All other values should be restored, so return the ready radio.
        Ok(DW3000 {
            ll: self.ll,
            seq: self.seq,
            state: Ready,
        })
    }*/
}
