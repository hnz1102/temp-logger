use log::*;
use std::{thread, time::Duration, sync::Arc, sync::Mutex};
use esp_idf_hal::i2c;
use ssd1306::{I2CDisplayInterface, prelude::*, Ssd1306};
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, ascii::FONT_5X8, MonoTextStyle},
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
            let small_style = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);
            display.clear();
            display.flush().unwrap();
            let wifibmp = Bmp::from_slice(include_bytes!("./img/wifirev.bmp")).unwrap();
            let wifi_img: Image<Bmp<BinaryColor>> = Image::new(&wifibmp, Point::new(108,1));

            // Battery BMP
            let bat0 = Bmp::from_slice(include_bytes!("./img/battery-0.bmp")).unwrap();
            let bat0_img: Image<Bmp<BinaryColor>> = Image::new(&bat0, Point::new(110,36));
            let bat20 = Bmp::from_slice(include_bytes!("./img/battery-20.bmp")).unwrap();
            let bat20_img: Image<Bmp<BinaryColor>> = Image::new(&bat20, Point::new(110,36));
            let bat40 = Bmp::from_slice(include_bytes!("./img/battery-40.bmp")).unwrap();
            let bat40_img: Image<Bmp<BinaryColor>> = Image::new(&bat40, Point::new(110,36));
            let bat60 = Bmp::from_slice(include_bytes!("./img/battery-60.bmp")).unwrap();
            let bat60_img: Image<Bmp<BinaryColor>> = Image::new(&bat60, Point::new(110,36));
            let bat80 = Bmp::from_slice(include_bytes!("./img/battery-80.bmp")).unwrap();
            let bat80_img: Image<Bmp<BinaryColor>> = Image::new(&bat80, Point::new(110,36));
            let bat100 = Bmp::from_slice(include_bytes!("./img/battery-100.bmp")).unwrap();
            let bat100_img: Image<Bmp<BinaryColor>> = Image::new(&bat100, Point::new(110,36));

            // Number BMP
            let n0 = Bmp::from_slice(include_bytes!("./img/n0.bmp")).unwrap();
            let n0_img: Image<Bmp<BinaryColor>> = Image::new(&n0, Point::zero());
            let n1 = Bmp::from_slice(include_bytes!("./img/n1.bmp")).unwrap();
            let n1_img: Image<Bmp<BinaryColor>> = Image::new(&n1, Point::zero());
            let n2 = Bmp::from_slice(include_bytes!("./img/n2.bmp")).unwrap();
            let n2_img: Image<Bmp<BinaryColor>> = Image::new(&n2, Point::zero());
            let n3 = Bmp::from_slice(include_bytes!("./img/n3.bmp")).unwrap();
            let n3_img: Image<Bmp<BinaryColor>> = Image::new(&n3, Point::zero());
            let n4 = Bmp::from_slice(include_bytes!("./img/n4.bmp")).unwrap();
            let n4_img: Image<Bmp<BinaryColor>> = Image::new(&n4, Point::zero());
            let n5 = Bmp::from_slice(include_bytes!("./img/n5.bmp")).unwrap();
            let n5_img: Image<Bmp<BinaryColor>> = Image::new(&n5, Point::zero());
            let n6 = Bmp::from_slice(include_bytes!("./img/n6.bmp")).unwrap();
            let n6_img: Image<Bmp<BinaryColor>> = Image::new(&n6, Point::zero());
            let n7 = Bmp::from_slice(include_bytes!("./img/n7.bmp")).unwrap();
            let n7_img: Image<Bmp<BinaryColor>> = Image::new(&n7, Point::zero());
            let n8 = Bmp::from_slice(include_bytes!("./img/n8.bmp")).unwrap();
            let n8_img: Image<Bmp<BinaryColor>> = Image::new(&n8, Point::zero());
            let n9 = Bmp::from_slice(include_bytes!("./img/n9.bmp")).unwrap();
            let n9_img: Image<Bmp<BinaryColor>> = Image::new(&n9, Point::zero());
            let cc = Bmp::from_slice(include_bytes!("./img/c.bmp")).unwrap();
            let cc_img: Image<Bmp<BinaryColor>> = Image::new(&cc, Point::new(88, 0));
            let dot = Bmp::from_slice(include_bytes!("./img/dot.bmp")).unwrap();
            let dot_img: Image<Bmp<BinaryColor>> = Image::new(&dot, Point::new(60, 0));
            let minus = Bmp::from_slice(include_bytes!("./img/minus.bmp")).unwrap();
            let minus_img: Image<Bmp<BinaryColor>> = Image::new(&minus, Point::zero());
            let mut digit_img = n0_img.translate(Point::new(0,0));

            let mut loopcount = 0;
            loop {
                let lck = txt.lock().unwrap();
                // Current Plate Temparature
                loopcount += 1;
                display.clear();
                let mut temp = lck.panel_temp;
                if temp != -999.0 {
                    dot_img.draw(&mut display).unwrap();                
                    cc_img.draw(&mut display).unwrap();
                    let mut digit_10 = 100.0;
                    let mut first_digit = true;
                    let mut pos_x = 0;
                    for digit in 0..=3 {
                        let num = (temp / digit_10) as i32;
                        match num {
                            0 => {
                                if !first_digit {
                                    digit_img = n0_img.translate(Point::new(pos_x, 0));
                                }
                            },
                            1 | -1 => {
                                digit_img = n1_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                            },
                            2 | -2 => {
                                digit_img = n2_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                            },
                            3 | -3 => {
                                digit_img = n3_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                            },
                            4 | -4 => {
                                digit_img = n4_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                            },
                            5 | -5 => {
                                digit_img = n5_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                            },
                            6 | -6 => {
                                digit_img = n6_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                            },
                            7 | -7 => {
                                digit_img = n7_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                            },
                            8 | -8 => {
                                digit_img = n8_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                            },
                            9 | -9 => {
                                digit_img = n9_img.translate(Point::new(pos_x, 0));
                                first_digit = false;
                            },
                            _ => {}
                        }
                        if temp < 0.0 && digit == 0 {
                            digit_img = minus_img.translate(Point::new(pos_x, 0));
                        }
                        digit_img.draw(&mut display).unwrap();
                        temp = temp - digit_10 * (num as f32);
                        pos_x += 20;
                        digit_10 /= 10.0;
                        if digit == 2 {
                            pos_x += 8;
                        }
                    }
                }
                else {
                    Text::new(&lck.panel_msg, Point::new(1, 20), style).draw(&mut display).unwrap();
                }
                match lck.status {
                    LoggingStatus::Start => {
                        match loopcount {
                            0..=5 | 10..=15 => {
                                Text::new("Logging..", Point::new(1, 55), style).draw(&mut display).unwrap();
                            },
                            _ => {},
                        }
                    },
                    LoggingStatus::Stop => {
                        Text::new("Stop", Point::new(1, 55), style).draw(&mut display).unwrap();
                    },
                }
                Text::new(&lck.pvol, Point::new(103, 62), small_style).draw(&mut display).unwrap();

                match lck.wifi {
                    WifiStatus::Disconnected => {
                        // match loopcount {
                        //     0..=5 | 10..=15 => {
                        //         wifi_img.draw(&mut display).unwrap();
                        //     },
                        //     _ => {},
                        // }
                    },
                    WifiStatus::Connected => {
                        wifi_img.draw(&mut display).unwrap();
                    },
                }
                if lck.volt < 3.7 {
                    bat0_img.draw(&mut display).unwrap();
                }
                else if lck.volt >= 3.7 && lck.volt < 3.8 {
                    bat20_img.draw(&mut display).unwrap();
                }
                else if lck.volt >= 3.8 && lck.volt < 3.9 {
                    bat40_img.draw(&mut display).unwrap();
                }
                else if lck.volt >= 3.9 && lck.volt < 4.0 {
                    bat60_img.draw(&mut display).unwrap();
                }
                else if lck.volt >= 4.0 && lck.volt < 4.1 {
                    bat80_img.draw(&mut display).unwrap();
                }
                else if lck.volt >= 4.1 {
                    bat100_img.draw(&mut display).unwrap();
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
        let mut lck = self.txt.lock().unwrap();
        lck.panel_temp = temp;
        lck.panel_msg = format!("{:.1}C", temp);
    }

    pub fn set_current_status(&mut self, status: LoggingStatus)
    {
        let mut lck= self.txt.lock().unwrap();
        lck.status = status;
    }

    pub fn set_wifi_status(&mut self, status: WifiStatus)
    {
        let mut lck= self.txt.lock().unwrap();
        lck.wifi = status;
    }

    pub fn set_err_message(&mut self, line : i32, msg: String)
    {
        let mut lck = self.txt.lock().unwrap();
        lck.panel_temp = -999.0;
        match line {
            1 => {
                lck.panel_msg = format!("{}", msg);
            },
            2 => {
                lck.target_msg = format!("T:{}", msg);
            }
            3 => {
                lck.setup_temp = format!("S:{}", msg);
            }
            _ => {
            }
        }
    }

    pub fn set_power_voltage(&mut self, volt: f32){
        let mut lck = self.txt.lock().unwrap();
        lck.pvol = format!("{:.2}V", volt);
        lck.volt = volt;
    }

}
