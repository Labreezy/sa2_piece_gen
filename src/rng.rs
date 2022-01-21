use std::num::Wrapping;

pub trait RngConsts {
    const MULT_COEFFICIENT: u32;
    const ADD_COEFFICIENT: u32;
}

pub struct PcRng;

impl RngConsts for PcRng {
    const MULT_COEFFICIENT: u32 = 0x000343FD;
    const ADD_COEFFICIENT: u32 = 0x00269EC3;
}

pub struct GcRng;

impl RngConsts for GcRng {
    const MULT_COEFFICIENT: u32 = 0x41C64E6D;
    const ADD_COEFFICIENT: u32 = 0x00003039;
}

#[derive(Clone, Copy, Debug)]
pub struct Rng {
    state: Wrapping<u32>,
}

impl Rng {
    pub fn new(seed: u32) -> Rng {
        Rng {
            state: Wrapping(seed)
        }
    }

    pub fn gen_val<R>(&mut self) -> u32
        where R: RngConsts,
    {
        // GC values are:
        // Mult: 0x41c64e6d
        // Add: 0x00003039
//        self.state = self.state * Wrapping(0x000343FD) + Wrapping(0x00269EC3);
        self.state = self.state * Wrapping(R::MULT_COEFFICIENT) + Wrapping(R::ADD_COEFFICIENT);
        (self.state.0 >> 0x10) & 0x7FFF
    }

    pub fn get_state(&self) -> u32 {
        self.state.0
    }
}
