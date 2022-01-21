pub mod rng;
pub mod emerald_manager;
pub mod vector;
pub mod stage_spec;
pub mod hint_lookup;

pub trait Platform {
    type SquareRoot: vector::Sqrt;
    type Consts: rng::RngConsts;
}

pub struct Gc;

impl Platform for Gc {
    type SquareRoot = vector::GcFp;
    type Consts = rng::GcRng;
}

pub struct Pc;

impl Platform for Pc {
    type SquareRoot = vector::PcFp;
    type Consts = rng::PcRng;
}
