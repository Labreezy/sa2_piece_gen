use std::ops::Sub;
use std::mem;
use std::f64;
use std::f32;

use serde_derive::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug)]
struct BaseAndDec {
    base: u32,
    dec: u32,
}

const FRSQRTE_EXPECTED: [BaseAndDec; 32] = [
    BaseAndDec{base: 0x3ffa000, dec: 0x7a4}, BaseAndDec{base: 0x3c29000, dec: 0x700},
    BaseAndDec{base: 0x38aa000, dec: 0x670}, BaseAndDec{base: 0x3572000, dec: 0x5f2},
    BaseAndDec{base: 0x3279000, dec: 0x584}, BaseAndDec{base: 0x2fb7000, dec: 0x524},
    BaseAndDec{base: 0x2d26000, dec: 0x4cc}, BaseAndDec{base: 0x2ac0000, dec: 0x47e},
    BaseAndDec{base: 0x2881000, dec: 0x43a}, BaseAndDec{base: 0x2665000, dec: 0x3fa},
    BaseAndDec{base: 0x2468000, dec: 0x3c2}, BaseAndDec{base: 0x2287000, dec: 0x38e},
    BaseAndDec{base: 0x20c1000, dec: 0x35e}, BaseAndDec{base: 0x1f12000, dec: 0x332},
    BaseAndDec{base: 0x1d79000, dec: 0x30a}, BaseAndDec{base: 0x1bf4000, dec: 0x2e6},
    BaseAndDec{base: 0x1a7e800, dec: 0x568}, BaseAndDec{base: 0x17cb800, dec: 0x4f3},
    BaseAndDec{base: 0x1552800, dec: 0x48d}, BaseAndDec{base: 0x130c000, dec: 0x435},
    BaseAndDec{base: 0x10f2000, dec: 0x3e7}, BaseAndDec{base: 0x0eff000, dec: 0x3a2},
    BaseAndDec{base: 0x0d2e000, dec: 0x365}, BaseAndDec{base: 0x0b7c000, dec: 0x32e},
    BaseAndDec{base: 0x09e5000, dec: 0x2fc}, BaseAndDec{base: 0x0867000, dec: 0x2d0},
    BaseAndDec{base: 0x06ff000, dec: 0x2a8}, BaseAndDec{base: 0x05ab800, dec: 0x283},
    BaseAndDec{base: 0x046a000, dec: 0x261}, BaseAndDec{base: 0x0339800, dec: 0x243},
    BaseAndDec{base: 0x0218800, dec: 0x226}, BaseAndDec{base: 0x0105800, dec: 0x20b},
];

const FRES_EXPECTED: [BaseAndDec; 32] = [
    BaseAndDec{base: 0x7ff800, dec: 0x3e1}, BaseAndDec{base: 0x783800, dec: 0x3a7},
    BaseAndDec{base: 0x70ea00, dec: 0x371}, BaseAndDec{base: 0x6a0800, dec: 0x340},
    BaseAndDec{base: 0x638800, dec: 0x313}, BaseAndDec{base: 0x5d6200, dec: 0x2ea},
    BaseAndDec{base: 0x579000, dec: 0x2c4}, BaseAndDec{base: 0x520800, dec: 0x2a0},
    BaseAndDec{base: 0x4cc800, dec: 0x27f}, BaseAndDec{base: 0x47ca00, dec: 0x261},
    BaseAndDec{base: 0x430800, dec: 0x245}, BaseAndDec{base: 0x3e8000, dec: 0x22a},
    BaseAndDec{base: 0x3a2c00, dec: 0x212}, BaseAndDec{base: 0x360800, dec: 0x1fb},
    BaseAndDec{base: 0x321400, dec: 0x1e5}, BaseAndDec{base: 0x2e4a00, dec: 0x1d1},
    BaseAndDec{base: 0x2aa800, dec: 0x1be}, BaseAndDec{base: 0x272c00, dec: 0x1ac},
    BaseAndDec{base: 0x23d600, dec: 0x19b}, BaseAndDec{base: 0x209e00, dec: 0x18b},
    BaseAndDec{base: 0x1d8800, dec: 0x17c}, BaseAndDec{base: 0x1a9000, dec: 0x16e},
    BaseAndDec{base: 0x17ae00, dec: 0x15b}, BaseAndDec{base: 0x14f800, dec: 0x15b},
    BaseAndDec{base: 0x124400, dec: 0x143}, BaseAndDec{base: 0x0fbe00, dec: 0x143},
    BaseAndDec{base: 0x0d3800, dec: 0x12d}, BaseAndDec{base: 0x0ade00, dec: 0x12d},
    BaseAndDec{base: 0x088400, dec: 0x11a}, BaseAndDec{base: 0x065000, dec: 0x11a},
    BaseAndDec{base: 0x041c00, dec: 0x108}, BaseAndDec{base: 0x020c00, dec: 0x106},
];

pub trait Sqrt {
    fn sqrt(val: f32) -> f32;
}

pub struct GcFp;

impl GcFp {
    fn frsqrte(val: f64) -> f64 {
        let integral: u64 = unsafe {
            mem::transmute(val)
        };
        let sign = integral & (1 << 63);
        let exponent = integral & (0x7FF << 52);
        let mantissa = integral & ((1 << 52) - 1);

        if exponent == 0 && mantissa == 0 {
            if sign == 0 {
                return f64::INFINITY;
            }
            else {
                return f64::NEG_INFINITY;
            }
        }

        if exponent == (0x7FF << 52) {
            if mantissa == 0 {
                if sign == 0 {
                    return 0.0;
                }
                else {
                    return f64::NAN;
                }
            }
            return 0.0 + val;
        }

        if sign != 0 {
            return f64::NAN;
        }

        let sqrt_exponent = ((0x3FF << 52) - ((exponent - (0x3FF << 52)) / 2)) & (0x7FF << 52);
        let mut sqrt = sign | sqrt_exponent;

        let i = mantissa >> 37;
        let index = (i >> 11) + if exponent & (1 << 52) != 0 { 16 } else { 0 };
        let entry = FRSQRTE_EXPECTED[index as usize];
        sqrt |= ((entry.base - entry.dec * (i as u32 % 2048)) as u64) << 26;

        unsafe {
            mem::transmute(sqrt)
        }
    }

    fn fres(val: f64) -> f64 {
        1.0 / val
    }

//    fn fres(val: f64) -> f64 {
//        let integral: u64 = unsafe {
//            mem::transmute(val)
//        };
//        let sign = integral & (1 << 63);
//        let exponent = integral & (0x7FF << 52);
//        let mantissa = integral & ((1 << 52) - 1);
//
//        if exponent == 0 && mantissa == 0 {
//            if sign == 0 {
//                return f64::INFINITY;
//            }
//            else {
//                return f64::NEG_INFINITY;
//            }
//        }
//
//        if exponent == (0x7FF << 52) {
//            if mantissa == 0 {
//                if sign == 0 {
//                    return 0.0;
//                }
//                else {
//                    return -0.0;
//                }
//            }
//            return 0.0 + val;
//        }
//
//        if exponent < 895 << 52 {
//            if sign == 0 {
//                return f64::MAX;
//            }
//            else {
//                return f64::MIN;
//            }
//        }
//
//        if exponent >= 1149 << 52 {
//            if sign == 0 {
//                return 0.0;
//            }
//            else {
//                return -0.0;
//            }
//        }
//
//        let reciprocal_exponent = (0x7FD << 52) - exponent;
//
//        let i = mantissa >> 37;
//        let entry = FRES_EXPECTED[(i >> 10) as usize];
//        let mut reciprocal = sign | reciprocal_exponent;
//        reciprocal |= ((entry.base - (entry.dec * (i as u32 % 1024) + 1)) as u64 / 2) << 29;
//
//        unsafe {
//            mem::transmute(reciprocal)
//        }
//    }
}

impl Sqrt for GcFp {
    fn sqrt(val: f32) -> f32 {
        Self::fres(Self::frsqrte(val as f64)) as f32
    }
}

pub struct PcFp;

impl Sqrt for PcFp {
    fn sqrt(val: f32) -> f32 {
        val.sqrt()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector {
    pub fn new(x: f32, y: f32, z: f32) -> Vector {
        Vector {
            x: x,
            y: y,
            z: z,
        }
    }

    pub fn cross(self, other: Vector) -> Vector {
        Vector {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn magnitude<F>(self) -> f32
        where F: Sqrt,
    {
        F::sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }

    pub fn distance<F>(self, other: Vector) -> f32
        where F: Sqrt,
    {
        let diff = self - other;
        let dist_squ = diff.x * diff.x + diff.y * diff.y + diff.z * diff.z;
        if dist_squ < 0.025 {
            0.0
        }
        else {
            F::sqrt(dist_squ)
        }
    }
}

impl Default for Vector {
    fn default() -> Self {
        Vector {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, other: Self) -> Self::Output {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
