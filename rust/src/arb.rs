use crate::arb_sys::*;
use inari::{interval, Interval};
use std::{mem::MaybeUninit, ops::Drop};

// Notes:
//
// - We always need to pass Arb pointers as `*_ptr` to Arb functions even if they expect `*_srcptr`,
//   due to: https://github.com/rust-lang/rust-bindgen/issues/1962
//
// - Arb is thread-safe, thus we implement `Send` and `Sync` for Arb types.
//   https://arblib.org/issues.html#thread-safety-and-caches

/// The precision of the `mag_t` type.
const MAG_BITS: u32 = 30;

/// Constants that correspond to the values of `arf_rnd_t`.
#[derive(Clone, Copy, Debug)]
enum ArfRound {
    // Down = 0,
    // Up = 1,
    Floor = 2,
    Ceil = 3,
    // Near = 4,
}

/// A wrapper for the `arf_t` type.
struct Arf(arf_struct);

impl Arf {
    /// Creates an `Arf` value initialized to be zero.
    pub fn new() -> Self {
        unsafe {
            let mut x = MaybeUninit::uninit();
            arf_init(x.as_mut_ptr());
            Self(x.assume_init())
        }
    }

    /// Returns an unsafe mutable pointer to the underlying `arf_t`.
    pub fn as_mut_ptr(&mut self) -> arf_ptr {
        &mut self.0
    }

    /// Returns an unsafe pointer to the underlying `arf_t`.
    pub fn as_ptr(&self) -> arf_srcptr {
        &self.0
    }

    /// Rounds `self` to a [`f64`] number using the given rounding mode.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_f64_round(&self, round: ArfRound) -> f64 {
        unsafe { arf_get_d(self.as_ptr() as arf_ptr, round as i32) }
    }
}

impl Drop for Arf {
    fn drop(&mut self) {
        unsafe {
            arf_clear(self.as_mut_ptr());
        }
    }
}

unsafe impl Send for Arf {}
unsafe impl Sync for Arf {}

/// A wrapper for the `arb_t` type.
pub struct Arb(arb_struct);

impl Arb {
    /// Creates an `Arb` interval initialized to be `[0 ± 0]`.
    pub fn new() -> Self {
        unsafe {
            let mut x = MaybeUninit::uninit();
            arb_init(x.as_mut_ptr());
            Self(x.assume_init())
        }
    }

    /// Returns an unsafe mutable pointer to the underlying `arb_t`.
    pub fn as_mut_ptr(&mut self) -> arb_ptr {
        &mut self.0
    }

    /// Returns an unsafe pointer to the underlying `arb_t`.
    pub fn as_ptr(&self) -> arb_srcptr {
        &self.0
    }

    /// Creates an `Arb` interval `[x ± 0]`.
    pub fn from_f64(x: f64) -> Self {
        let mut y = Self::new();
        unsafe {
            arb_set_d(y.as_mut_ptr(), x);
        }
        y
    }

    /// Creates an `Arb` interval that encloses `x`.
    pub fn from_interval(x: Interval) -> Self {
        let mut y = Self::new();
        if !x.is_common_interval() {
            unsafe {
                arb_zero_pm_inf(y.as_mut_ptr());
            }
        } else {
            // Construct an `Arb` interval faster and more precisely than
            // using `arb_set_interval_arf`.

            let mid = x.mid();
            unsafe {
                arf_set_d(&mut y.0.mid, mid);
            }

            let rad = x.rad();
            if rad != 0.0 {
                let (man, mut exp) = frexp(rad);
                let mut man = (man * (1 << MAG_BITS) as f64).ceil() as u32;
                if man == 1 << MAG_BITS {
                    // Restrict the mantissa within 30 bits:
                    //   100...000 ≤ `man` ≤ 111...111 (30 1's).
                    man = 1 << (MAG_BITS - 1);
                    exp += 1;
                }
                // For safer construction, see `mag_set_ui_2exp_si`.
                // https://github.com/fredrik-johansson/arb/blob/master/mag/set_ui_2exp_si.c
                y.0.rad.exp = exp.into();
                y.0.rad.man = man.into();
            }
        }
        y
    }

    /// Returns an [`Interval`] that encloses `self`.
    #[allow(clippy::wrong_self_convention)]
    pub fn to_interval(&self) -> Interval {
        let mut a = Arf::new();
        let mut b = Arf::new();
        unsafe {
            arb_get_interval_arf(
                a.as_mut_ptr(),
                b.as_mut_ptr(),
                self.as_ptr() as arb_ptr,
                f64::MANTISSA_DIGITS.into(),
            );
        }
        interval!(
            a.to_f64_round(ArfRound::Floor),
            b.to_f64_round(ArfRound::Ceil)
        )
        .unwrap_or(Interval::ENTIRE) // [+∞ ± c], [-∞ ± c] or [NaN ± c]
    }
}

impl Drop for Arb {
    fn drop(&mut self) {
        unsafe {
            arb_clear(self.as_mut_ptr());
        }
    }
}

unsafe impl Send for Arb {}
unsafe impl Sync for Arb {}

/// A wrapper for the `acb_t` type.
pub struct Acb(acb_struct);

impl Acb {
    /// Creates an `Acb` interval initialized to be `[0 ± 0] + [0 ± 0]i`.
    pub fn new() -> Self {
        unsafe {
            let mut x = MaybeUninit::uninit();
            acb_init(x.as_mut_ptr());
            Self(x.assume_init())
        }
    }

    /// Returns an unsafe mutable pointer to the underlying `acb_t`.
    pub fn as_mut_ptr(&mut self) -> acb_ptr {
        &mut self.0
    }

    /// Returns an unsafe pointer to the underlying `acb_t`.
    pub fn as_ptr(&self) -> acb_srcptr {
        &self.0
    }

    /// Returns the real part of `self`.
    pub fn real(&self) -> Arb {
        let mut x = Arb::new();
        unsafe {
            acb_get_real(x.as_mut_ptr(), self.as_ptr() as acb_ptr);
        }
        x
    }
}

impl Drop for Acb {
    fn drop(&mut self) {
        unsafe {
            acb_clear(self.as_mut_ptr());
        }
    }
}

impl From<Arb> for Acb {
    fn from(mut x: Arb) -> Self {
        let mut z = Acb::new();
        unsafe {
            acb_set_arb(z.as_mut_ptr(), x.as_mut_ptr());
        }
        z
    }
}

unsafe impl Send for Acb {}
unsafe impl Sync for Acb {}

// A copy-paste of https://github.com/rust-lang/libm/blob/master/src/math/frexp.rs
fn frexp(x: f64) -> (f64, i32) {
    let mut y = x.to_bits();
    let ee = ((y >> 52) & 0x7ff) as i32;

    if ee == 0 {
        if x != 0.0 {
            let x1p64 = f64::from_bits(0x43f0000000000000);
            let (x, e) = frexp(x * x1p64);
            return (x, e - 64);
        }
        return (x, 0);
    } else if ee == 0x7ff {
        return (x, 0);
    }

    let e = ee - 0x3fe;
    y &= 0x800fffffffffffff;
    y |= 0x3fe0000000000000;
    (f64::from_bits(y), e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use inari::{const_interval, Interval};

    #[test]
    fn inclusion_property() {
        let xs = [
            Interval::EMPTY,
            const_interval!(0.0, 0.0),
            const_interval!(1.0, 1.0),
            Interval::PI,
            const_interval!(0.0, f64::INFINITY),
            const_interval!(f64::NEG_INFINITY, 0.0),
            Interval::ENTIRE,
            // The case where rounding up the interval radius (`mag_t`) produces a carry:
            // As opposed to `f64`, the hidden bit is not used in the mantissa of a `mag_t`.
            //         a =  0.0₂
            //         b =  0.111...111 1₂ × 2^1
            //                ^^^^^^^^^^^ 31 1's (1-bit larger than what can fit in the mantissa)
            // (b - a)/2 =  0.111...111 1₂ × 2^0
            //       rad =  1.000...00₂    × 2^0  <- Round the mantissa of (b - a)/2 up to
            //           =  0.100...000₂   × 2^1     the nearest 30-bit number. (produces a carry)
            //                ^^^^^^^^^ the mantissa of a `mag_t` (30-bit)
            const_interval!(0.0, 1.9999999990686774),
        ];
        for x in &xs {
            let y = Arb::from_interval(*x).to_interval();
            assert!(x.subset(y));
        }
    }
}
