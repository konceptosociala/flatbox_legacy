use std::cmp::min;
use std::ops::{Add, Sub, AddAssign, SubAssign};
use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Volume(u8);

impl Default for Volume {
    fn default() -> Self {
        Volume(100)
    }
}

impl Volume {
    pub fn new<U>(value: U) -> Volume 
    where
        U: Into<u8>
    {
        Volume(min(value.into(), 100))
    }
    
    pub fn set_volume<U>(&mut self, value: U)
    where
        U: Into<u8>
    {
        self.0 = min(value.into(), 100);
    }
}

impl<U: Into<u8>> Add<U> for Volume {
    type Output = Self;

    fn add(self, value: U) -> Self {
        let mut volume = self;
        volume.set_volume(self.0 + value.into());
        volume
    }
}

impl<U: Into<u8>> AddAssign<U> for Volume {
    fn add_assign(&mut self, value: U){
        self.set_volume(self.0 + value.into ());
    }
}

impl<U: Into<u8>> Sub<U> for Volume {
    type Output = Self;

    fn sub(self, value: U) -> Self {
        let mut volume = self;
        volume.set_volume(self.0 - value.into());
        volume
    }
}

impl<U: Into<u8>> SubAssign<U> for Volume {
    fn sub_assign(&mut self, value: U){
        self.set_volume(self.0 - value.into());
    }
}