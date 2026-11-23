//! Deterministic seeded PRNG (SplitMix64) for reproducible randomized algorithms.

use serde::{Deserialize, Serialize};

/// Deterministic 64-bit SplitMix64 PRNG.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicPrng {
    state: u64,
}

impl DeterministicPrng {
    /// Construct a PRNG from an explicit 64-bit seed.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Generate the next uniform `u64`.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Generate a uniform `i64` in `[low, high]` inclusive.
    pub fn gen_range_i64(&mut self, low: i64, high: i64) -> i64 {
        if low >= high {
            return low;
        }
        let span = (high as i128 - low as i128 + 1) as u128;
        let r = (self.next_u64() as u128) % span;
        (low as i128 + r as i128) as i64
    }

    /// Generate a uniform `f64` in `[0.0, 1.0)`.
    pub fn gen_unit_f64(&mut self) -> f64 {
        let bits = self.next_u64() >> 11;
        (bits as f64) * (1.0 / ((1u64 << 53) as f64))
    }
}
