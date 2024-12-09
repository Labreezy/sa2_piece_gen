use std::ops::Sub;
use std::num::Wrapping;
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

pub trait PlatformMath {
    fn sqrt(val: f32) -> f32;
    fn cross(v1: Vector, v2: Vector) -> Vector;
    fn magnitude(v: Vector) -> f32;
}

pub struct GcFp;

impl GcFp {
    pub fn frsqrte(val: f64) -> f64 {
        let integral = val.to_bits();
        let sign = integral & (1 << 63);
        let mut exponent = integral & (0x7FFu64 << 52);
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

        // Get exponent LSB before we modify it
        let exponent_lsb = exponent & (1 << 52) ^ (1 << 52);

        // Divide exponent by 2?
        exponent = (Wrapping(0x3FF << 52) - Wrapping((Wrapping(exponent) - Wrapping(0x3FE << 52)).0 / 2)).0 & (0x7FF << 52);

        // Get entry in lerp table
        let idx = (exponent_lsb | mantissa) >> 37;
        let entry = FRSQRTE_EXPECTED[(idx / 2048) as usize];

        // Start making the result since we already have the sign/exponent
        let mut sqrt = sign | exponent;

        // Interpolate from base to next entry using dec(lination?) as the slope
        sqrt |= ((entry.base - entry.dec * (idx as u32 % 2048)) as u64) << 26;

        f64::from_bits(sqrt)
    }

    pub fn fres(val: f64) -> f64 {
        let integral = val.to_bits();
        let sign = integral & (1 << 63);
        let mut exponent = integral & (0x7FFu64 << 52);
        let mantissa = integral & ((1 << 52) - 1);

        if exponent == 0 && mantissa == 0 {
            if sign == 0 {
                return f64::INFINITY;
            }
            else {
                return f64::NEG_INFINITY;
            }
        }

        if exponent < (895 << 52) {
            return f64::MAX;
        }

        if exponent >= (1149 << 52) {
            return 0.0f64;
        }

        // Negate exponent
        exponent = (0x7FD << 52) - exponent;

        // Get entry in lerp table
        let idx = mantissa >> 37;
        let entry = FRES_EXPECTED[(idx / 1024) as usize];

        // Start making the result since we already have the sign/exponent
        let mut inv = sign | exponent;

        // Interpolate from base to next entry using dec(lination?) as the slope
        inv |= ((entry.base - (entry.dec * (idx as u32 % 1024) + 1) / 2) as u64) << 29;

        f64::from_bits(inv)
    }

    pub fn fmuls(a: f32, c: f32) -> f32 {
        (a as f64 * c as f64) as f32
    }

    fn fmadds(a: f32, c: f32, b: f32) -> f32 {
        (a as f64 * c as f64 + b as f64) as f32
    }
}

impl PlatformMath for GcFp {
    fn sqrt(val: f32) -> f32 {
        Self::fres(Self::frsqrte(val as f64)) as f32
    }

    fn cross(v1: Vector, v2: Vector) -> Vector {
        Vector {
            x: Self::fmadds(v2.z, v1.y, -Self::fmuls(v2.y, v1.z)),
            y: Self::fmadds(v2.x, v1.z, -Self::fmuls(v2.z, v1.x)),
            z: Self::fmadds(v2.y, v1.x, -Self::fmuls(v2.x, v1.y)),
        }
    }

    fn magnitude(v: Vector) -> f32 {
        let sum_square = Self::fmuls(v.x, v.x) as f64 + v.y as f64 * v.y as f64 + v.z as f64 * v.z as f64;
        Self::fres(Self::frsqrte(sum_square)) as f32
    }
}

pub struct PcFp;

impl PlatformMath for PcFp {
    fn sqrt(val: f32) -> f32 {
        val.sqrt()
    }

    fn cross(v1: Vector, v2: Vector) -> Vector {
        Vector {
            x: v1.y * v2.z - v1.z * v2.y,
            y: v1.z * v2.x - v1.x * v2.z,
            z: v1.x * v2.y - v1.y * v2.x,
        }
    }

    fn magnitude(v: Vector) -> f32 {
        Self::sqrt(v.x * v.x + v.y * v.y + v.z * v.z)
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

    pub fn cross<F>(self, other: Vector) -> Vector 
        where F: PlatformMath,
    {
        F::cross(self, other)
    }

    pub fn magnitude<F>(self) -> f32
        where F: PlatformMath,
    {
        F::magnitude(self)
    }

    pub fn distance<F>(self, other: Vector) -> f32
        where F: PlatformMath,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_fn_f64_eq_from_bits<F>(func: F, input: u64, output: u64)
    where
        F: Fn(f64) -> f64,
    {
        assert_eq!(func(f64::from_bits(input)).to_bits(), output);
    }

    #[test]
    fn test_frsqrte() {
        test_fn_f64_eq_from_bits(GcFp::frsqrte, 0x3ea8792d45540000, 0x40924c1090000000);
        test_fn_f64_eq_from_bits(GcFp::frsqrte, 0x3f00293b64599c80, 0x406683d560000000);
        test_fn_f64_eq_from_bits(GcFp::frsqrte, 0x0000000000000000, 0x7ff0000000000000);
        test_fn_f64_eq_from_bits(GcFp::frsqrte, 0x3ef4d01b63e44000, 0x406c0ed800000000);
        test_fn_f64_eq_from_bits(GcFp::frsqrte, 0x3e6b34191b000000, 0x40b15a8c80000000);
    }

    #[test]
    fn test_fres() {
        test_fn_f64_eq_from_bits(GcFp::fres, 0x40b15a8c80000000, 0x3f2d8186c0000000);
        test_fn_f64_eq_from_bits(GcFp::fres, 0x7ff0000000000000, 0x0000000000000000);
        test_fn_f64_eq_from_bits(GcFp::fres, 0x408103dcfc000000, 0x3f5e16cc20000000);
        test_fn_f64_eq_from_bits(GcFp::fres, 0x4059e10cb8000000, 0x3f83c8ea80000000);
        test_fn_f64_eq_from_bits(GcFp::fres, 0x4054ca52ec000000, 0x3f88a0eee0000000);
    }
}