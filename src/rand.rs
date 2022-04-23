use core::num::Wrapping;

pub struct Rand {
    next: Wrapping<u32>,
}

impl Rand {
    pub fn new(seed: u32) -> Self{
        Self {
            next: Wrapping(seed),
        }
    }

    pub fn next(&mut self) -> u32 {
        self.next = self.next * Wrapping(1103515245) + Wrapping(12345);
        return ((self.next/Wrapping(65536)) % Wrapping(32768)).0;
    }
}
