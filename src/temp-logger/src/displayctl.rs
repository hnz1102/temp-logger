use log::*;
use std::{thread, time::Duration, sync::Arc, sync::Mutex};
use esp_idf_hal::i2c;
use ssd1306::{I2CDisplayInterface, prelude::*, Ssd1306};
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    image::Image,
    pixelcolor::{BinaryColor},
    text::{Text},
    geometry::Point,
    prelude::*,
};
use tinybmp::Bmp;

pub enum LoggingStatus {
    Start,
    Stop,
}

pub enum WifiStatus {
    Connected,
    Disconnected,
}

type DISPLAYIF<'d> = i2c::I2cDriver<'static>;

struct DisplayText {
    panel_temp: f32,
    panel_msg: String,
    // target_temp: f32,
    target_msg: String,
    setup_temp: String,
    // current_power: u32,
    status: LoggingStatus,
    wifi: WifiStatus,
    // menu: String,
    pvol: String,
    volt: f32,
}

pub struct DisplayPanel {
    txt: Arc<Mutex<DisplayText>>
}

impl DisplayPanel {
    pub fn new() -> DisplayPanel {
        DisplayPanel { txt: Arc::new(Mutex::new(
            DisplayText {panel_temp: 0.0,
                         panel_msg: "".to_string(),
                        //  target_temp: 0.0,
                         target_msg: "".to_string(),
                         setup_temp: "".to_string(),
                        //  current_power: 0,
                         status: LoggingStatus::Stop,
                         wifi: WifiStatus::Disconnected,
                        //  menu: "".to_string(),
                         pvol: "".to_string(),
                         volt: 0.0,
                     })) }
    }

    pub fn start(&mut self, i2c : DISPLAYIF )
    {
        let txt = self.txt.clone();
        let _th = thread::spawn(move || {
            info!("Start Display Thread.");
            let interface = I2CDisplayInterface::new(i2c);        
            let mut display = Ssd1306::new(interface, 
                DisplaySize128x64,
                ssd1306::prelude::DisplayRotation::Rotate180)
                .into_buffered_graphics_mode();        
            display.init().unwrap();
            let style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
            display.clear();
            display.flush().unwrap();
            let wifibmp = Bmp::from_slice(include_bytes!("./img/wifirev.bmp"))
                .expect("Failed to load BMP image");
            let wifiimg: Image<Bmp<BinaryColor>> = Image::new(&wifibmp, Point::zero());
            let wifitrans = wifiimg.translate(Point::new(95,1));

            // Battery BMP
            let bat0 = Bmp::from_slice(include_bytes!("./img/battery-0.bmp"))
                .expect("Failed to load BMP image");
            let bat0img: Image<Bmp<BinaryColor>> = Image::new(&bat0, Point::zero());
            let bat0tr = bat0img.translate(Point::new(95,40));
            let bat20 = Bmp::from_slice(include_bytes!("./img/battery-20.bmp"))
                .expect("Failed to load BMP image");
            let bat20img: Image<Bmp<BinaryColor>> = Image::new(&bat20, Point::zero());
            let bat20tr = bat20img.translate(Point::new(95,40));
            let bat40 = Bmp::from_slice(include_bytes!("./img/battery-40.bmp"))
                .expect("Failed to load BMP image");
            let bat40img: Image<Bmp<BinaryColor>> = Image::new(&bat40, Point::zero());
            let bat40tr = bat40img.translate(Point::new(95,40));
            let bat60 = Bmp::from_slice(include_bytes!("./img/battery-60.bmp"))
                .expect("Failed to load BMP image");
            let bat60img: Image<Bmp<BinaryColor>> = Image::new(&bat60, Point::zero());
            let bat60tr = bat60img.translate(Point::new(95,40));
            let bat80 = Bmp::from_slice(include_bytes!("./img/battery-80.bmp"))
                .expect("Failed to load BMP image");
            let bat80img: Image<Bmp<BinaryColor>> = Image::new(&bat80, Point::zero());
            let bat80tr = bat80img.translate(Point::new(95,40));
            let bat100 = Bmp::from_slice(include_bytes!("./img/battery-100.bmp"))
                .expect("Failed to load BMP image");
            let bat100img: Image<Bmp<BinaryColor>> = Image::new(&bat100, Point::zero());
            let bat100tr = bat100img.translate(Point::new(95,40));

            let mut loopcount = 0;
            loop {
                let lck = txt.lock().unwrap();
                // Current Plate Temparature
                loopcount += 1;
                display.clear();
                Text::new(&lck.panel_msg, Point::new(1, 20), style).draw(&mut display).unwrap();
                match lck.status {
                    LoggingStatus::Start => {
                        match loopcount {
                            0..=5 | 10..=15 => {
                                Text::new("Logging..", Point::new(1, 40), style).draw(&mut display).unwrap();
                            },
                            _ => {},
                        }
                    },
                    LoggingStatus::Stop => {
                        Text::new("Stop", Point::new(1, 40), style).draw(&mut display).unwrap();
                    },
                }
                Text::new(&lck.pvol, Point::new(1, 60), style).draw(&mut display).unwrap();

                match lck.wifi {
                    WifiStatus::Disconnected => {
                        // match loopcount {
                        //     0..=5 | 10..=15 => {
                        //         wifitrans.draw(&mut display).unwrap();
                        //     },
                        //     _ => {},
                        // }
                    },
                    WifiStatus::Connected => {
                        wifitrans.draw(&mut display).unwrap();
                    },
                }
                if lck.volt < 3.7 {
                    bat0tr.draw(&mut display).unwrap();
                }
                else if lck.volt >= 3.7 && lck.volt < 3.8 {
                    bat20tr.draw(&mut display).unwrap();
                }
                else if lck.volt >= 3.8 && lck.volt < 3.9 {
                    bat40tr.draw(&mut display).unwrap();
                }
                else if lck.volt >= 3.9 && lck.volt < 4.0 {
                    bat60tr.draw(&mut display).unwrap();
                }
                else if lck.volt >= 4.0 && lck.volt < 4.1 {
                    bat80tr.draw(&mut display).unwrap();
                }
                else if lck.volt >= 4.1 {
                    bat100tr.draw(&mut display).unwrap();
                }
                if loopcount == 15 {
                    loopcount = 0;
                }
                display.flush().unwrap();
                drop(lck);                
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    pub fn set_panel_temp(&mut self, temp: f32)
    {
        let mut lcktxt = self.txt.lock().unwrap();
        lcktxt.panel_temp = temp;
        lcktxt.panel_msg = format!("{:.1}C", temp);
    }

    pub fn set_current_status(&mut self, status: LoggingStatus)
    {
        let mut lcktxt= self.txt.lock().unwrap();
        lcktxt.status = status;
    }

    pub fn set_wifi_status(&mut self, status: WifiStatus)
    {
        let mut lcktxt= self.txt.lock().unwrap();
        lcktxt.wifi = status;
    }

    pub fn set_err_message(&mut self, line : i32, msg: String)
    {
        let mut lcktxt = self.txt.lock().unwrap();
        match line {
            1 => {
                lcktxt.panel_msg = format!("{}", msg);
            },
            2 => {
                lcktxt.target_msg = format!("T:{}", msg);
            }
            3 => {
                lcktxt.setup_temp = format!("S:{}", msg);
            }
            _ => {
            }
        }
    }

    pub fn set_power_voltage(&mut self, volt: f32){
        let mut lcktxt = self.txt.lock().unwrap();
        lcktxt.pvol = format!("{:.2}V", volt);
        lcktxt.volt = volt;
    }

}
