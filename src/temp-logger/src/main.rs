

use std::{thread, time::Duration};
use esp_idf_hal::{gpio::*, prelude::*, spi, i2c};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use embedded_hal::spi::MODE_0;
use log::*;
use std::time::SystemTime;
use esp_idf_hal::adc::config::Config as AdcConfig;
use esp_idf_hal::adc::AdcChannelDriver;
use esp_idf_hal::adc::AdcDriver;
use esp_idf_hal::adc::Atten11dB;

mod pushswitch;
mod displayctl;
mod templogs;
mod wifi;
mod transfer;

use pushswitch::PushSwitch;
use displayctl::{DisplayPanel, LoggingStatus, WifiStatus};
use templogs::{TempRecord, TempLog};
use transfer::Transfer;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    http_server: &'static str,
}

struct Temperature {
    _ta: f32,
    tr: f32,
    msg: String,
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Peripherals Initialize
    let peripherals = Peripherals::take().unwrap();
    
    // PWM
    let timer_driver = LedcTimerDriver::new(peripherals.ledc.timer0, &TimerConfig::default().frequency(50.Hz().into())).unwrap();
    let mut pwm_driver = LedcDriver::new(peripherals.ledc.channel0, timer_driver, peripherals.pins.gpio10).unwrap();
    pwm_driver.set_duty(0).expect("Set duty failure");
    // SPI Temperature
    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio3;
    let sdi  = peripherals.pins.gpio4;
    let sdo_not_used = peripherals.pins.gpio1;
    let cs1 = peripherals.pins.gpio5;
    let spi_config = spi::SpiConfig::new().baudrate(1.MHz().into()).data_mode(MODE_0);
    let spi_driver = spi::SpiDriver::new(
        spi,
        sclk,
        sdo_not_used,
        Some(sdi),
        spi::Dma::Disabled,
    ).unwrap();
    
    let spi_shared_device = spi::SpiSharedDeviceDriver::new(spi_driver, &spi_config).unwrap();
    let mut spi1 = spi::SpiSoftCsDeviceDriver::new(&spi_shared_device, cs1, esp_idf_hal::gpio::Level::High).unwrap();

    // Display
    let i2c = peripherals.i2c0;
    let scl = peripherals.pins.gpio8;
    let sda = peripherals.pins.gpio9;
    let config = i2c::I2cConfig::new().baudrate(1.MHz().into());
    let i2c = i2c::I2cDriver::new(i2c, sda, scl, &config)?;
    let mut dp = DisplayPanel::new();
    dp.start(i2c);

    // PushSW
    let startstop_pin   = peripherals.pins.gpio20;
    let send_pin        = peripherals.pins.gpio21;
    let startstop_sig   = Box::new(PinDriver::input(startstop_pin)?);
    let send_sig        = Box::new(PinDriver::input(send_pin)?);
    let mut psw         = PushSwitch::new();
    psw.start(startstop_sig, send_sig);
    let mut startstop_led   = PinDriver::input_output(peripherals.pins.gpio6)?;
    let mut sending_led     = PinDriver::input_output(peripherals.pins.gpio7)?;
    startstop_led.set_low()?;
    sending_led.set_low()?;

    // Temperature Logs
    let mut tlogs = TempRecord::new();

    // WiFi
    let wifi_enable : bool;
    let wifi = wifi::wifi_connect(peripherals.modem, CONFIG.wifi_ssid, CONFIG.wifi_psk);
    match wifi {
        Ok(_) => { wifi_enable = true; },
        Err(e) => { info!("{:?}", e); wifi_enable = false }
    }
    let mut txd =  Transfer::new(CONFIG.http_server.to_string());
    txd.start()?;
    
    // ADC
    let mut adc = AdcDriver::new(peripherals.adc1, &AdcConfig::new().calibration(true))?;
    let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<'_, Gpio2, Atten11dB<_>> =
        AdcChannelDriver::new(peripherals.pins.gpio2)?;

    // loop
    let mut count : u32 = 0;
    let mut logging_start = false;
    let mut sending_start = false;
    let mut now = SystemTime::now();
    loop {
        thread::sleep(Duration::from_millis(100));
        let start_stop_btn = psw.get_gpio_state(20);
        let sending_btn = psw.get_gpio_state(21);
        if start_stop_btn == true {
            if logging_start == true {
                // to Stop
                logging_start = false;
                dp.set_current_status(LoggingStatus::Stop);
                tlogs.dump();
                tlogs.clear();
                startstop_led.set_low()?;                
            }
            else {
                // to Start
                logging_start = true;
                info!("Logging Start..");
                now = SystemTime::now();
                dp.set_current_status(LoggingStatus::Start);
                startstop_led.set_high()?;
            }
        }
        if sending_btn == true {
            if sending_start == true {
                // Stop Sending
                sending_start = false;
                sending_led.set_low()?;
                if logging_start == true {
                    dp.set_current_status(LoggingStatus::Start);
                }
                else {
                    dp.set_current_status(LoggingStatus::Stop);
                }
            }
            else {
                // Start Sending Data
                sending_start = true;
                sending_led.set_high()?;
            }
        }
        if wifi_enable == false{
            dp.set_wifi_status(WifiStatus::Disconnected);
        }
        else {
            dp.set_wifi_status(WifiStatus::Connected);
        }
        count += 1;
        if count % 10 != 0 {
            continue;
        }
        // Read Power Voltage mV
       let pvol =  adc.read(&mut adc_pin).unwrap() as f32 * 2.0 / 1000.0;
       dp.set_power_voltage(pvol);

        // Read Temperature
        let mut temp = [0u8; 4];
        let mut data = TempLog::default();
        data.vol = pvol;
        match spi1.read(&mut temp) {
            Ok(_v) => {
                let temp1 = get_temp(&temp);
                match temp1 {
                    Ok(v) => {
                        dp.set_panel_temp(v.tr);
                        data.pt = v.tr;
                    }
                    Err(e) => {
                        dp.set_err_message(1, e.msg);
                        data.pt = -99.99;
                    }
                }        
            },
            Err(e) => { info!("{:?}", e); }
        }
        data.clock = now.elapsed().unwrap().as_millis() as u32;
        if logging_start {
            tlogs.record(data);
        }
        if wifi_enable == true && sending_start == true {
            let logs = tlogs.get_all_data();
            let txcount = txd.set_transfer_data(logs);
            if txcount > 0 {
                tlogs.remove_data(txcount);
            }
        }
    }
}

fn get_temp(temp: & [u8; 4]) -> Result<Temperature, Temperature>
{
    let msg;
    match temp[3] & 0x7 {
        0x01 => { msg = "OC ERROR"  },
        0x02 => { msg = "SCG ERROR" },
        0x04 => { msg = "SCV ERROR" },
        _ => {
            let tempval : i32 = (((temp[0] as i8) as i32) << 6) | ((temp[1] >> 2) as i32);
            let intempval : i32 = (((temp[2] as i8) as i32) << 4) | ((temp[3] >> 4) as i32);
            let ta = (intempval as f32) * 0.0625;
            let tr = (tempval as f32) * 0.25;
            return Ok(Temperature { _ta: ta, tr: tr, msg: "OK".to_string() });
        }
    }
    Err(Temperature { _ta: 0.0, tr: 0.0, msg: msg.to_string() })
}

