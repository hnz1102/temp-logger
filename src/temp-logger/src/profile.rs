use log::*;

#[derive(Debug, Clone, Copy)]
pub struct Profile {
    pub temp: u32,
    pub dura: u32, 
}

pub struct TempTransition {
    pub profile: Profile,
    pub chtime: u32,
    pub pass: bool,
    pub set: bool,
}

pub struct ProfileTable {
    db: Vec<TempTransition>,
}

impl ProfileTable {
    pub fn new() -> Self  {
        ProfileTable { db: Vec::new() }
    }

    pub fn add_point(&mut self, prof: Profile)
    {
        let nt =  TempTransition { profile: prof, chtime: 0, pass: false, set: false };
        self.db.push(nt);
    }

    pub fn get_profile(&mut self) -> &mut [TempTransition] {
        &mut (self.db[..])
    }

    pub fn dump(&self)
    {
        info!("temp,duration,chtime,pass");
        for it in &self.db {
            info!("{},{},{},{},{}", it.profile.temp, it.profile.dura, it.chtime, it.pass, it.set);
        }
    }

}
