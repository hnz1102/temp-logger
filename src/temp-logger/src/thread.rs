use log::*;
use std::{thread, time::Duration, sync::Arc, sync::Mutex};
use esp_idf_hal::gpio::PinDriver;

struct PWMControlConfig {
    set_width: u32,
    set_duration: u32,
}

pub struct PWMControl {
    config: Arc<Mutex<PWMControlConfig>>
}

impl Drop for PWMControl {
    fn drop(&mut self){
        info!("Destructed PWM_Config");
    }
}

impl PWMControl {
    pub fn new() -> Self {
        Self { config: Arc::new(Mutex::new(PWMControlConfig {set_width: 0, set_duration: 0})) }
    }

    pub fn start_control<T>(&mut self, ticks: u64, pin: T) -> anyhow::Result<()>
    where
        T: esp_idf_hal::gpio::Pin + Send + 'static + esp_idf_hal::gpio::OutputPin,
    {
        let cfg = self.config.clone();
        let _th = thread::spawn(move || {
            info!("Start Power Control Thread.");
            let mut count : u32 = 0;
            let mut heat_power = PinDriver::output(pin).unwrap();
            heat_power.set_low().expect("Set failed.");
        
            loop {
                thread::sleep(Duration::from_millis(ticks));
                let locked_cfg = cfg.lock().unwrap();
                info!("count {:?} set_duration {:?} set_width {:?}", count, locked_cfg.set_duration, locked_cfg.set_width);
                count += 1;
                if count >= locked_cfg.set_duration { count = 0; }
                if count >= locked_cfg.set_width {
                    heat_power.set_low().expect("Set failed.");
                }
                else {
                    heat_power.set_high().expect("Set failed.");
                }
                drop(locked_cfg);
            }
        });
        Ok(())
    }

    pub fn set_pwm(&mut self, width: u32, duration: u32)
    {
        let mut locked_config = self.config.lock().expect("lock failed");
        locked_config.set_width = width;
        locked_config.set_duration = duration;
    }

}
