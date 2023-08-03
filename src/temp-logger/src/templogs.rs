use log::*;

pub struct TempLog {
    pub pt: f32,
    pub tt: f32,
    pub clock: u32,
    pub pwr: u32,
    pub vol: f32, 
}

impl TempLog {
    pub fn default() -> Self {
        TempLog { pt: 0.0, tt: 0.0, clock: 0, pwr: 0, vol: 0.0 }
    }
}


pub struct TempRecord {
    rec: Vec<TempLog>,
}

impl TempRecord {
    pub fn new() -> TempRecord {
        TempRecord { rec: Vec::new() }
    }

    pub fn record(&mut self, data: TempLog)
    {
        self.rec.push(data);
    }

    pub fn dump(&self)
    {
        info!("time,plate,target,power");
        for it in &self.rec {
            info!("{},{},{},{}", it.clock, it.pt, it.tt, it.pwr);
        } 
    }

    pub fn clear(&mut self)
    {
        self.rec.clear()
    }

    pub fn get_all_data(&self) -> &Vec<TempLog> {
        &self.rec
    }

    pub fn remove_data(&mut self, size : usize){
        let mut num = size;
        if self.rec.len() < size {
            num = self.rec.len();
        }       
        let _ = &self.rec.drain(0..num);
    }

}

