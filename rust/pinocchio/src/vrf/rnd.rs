//! Deterministic helpers for deriving typed random values from a 32-byte VRF seed.
//!
//! These mirror the canonical `ephemeral_vrf_sdk::rnd` helpers byte-for-byte so a
//! pinocchio program consuming a VRF callback derives the same values as a program
//! built against the reference SDK.

/// Generates a random `u8` value from a 32-byte random seed.
pub fn random_u8(bytes: &[u8; 32]) -> u8 {
    bytes[30]
}

/// Generates a random `u8` value within the inclusive range `[min_value, max_value]`.
///
/// To avoid modulo bias, this scans the seed bytes in reverse looking for a value
/// below an evenly-divisible threshold. If none is found it falls back to the last
/// byte (slightly biased, but a rare case).
pub fn random_u8_with_range(bytes: &[u8; 32], min_value: u8, max_value: u8) -> u8 {
    let range = max_value as u16 - min_value as u16 + 1;
    let threshold = 256 / range * range;

    // Try to find a byte that, when mapped, gives an unbiased result
    for &b in bytes.iter().rev() {
        if (b as u16) < threshold {
            return (min_value as u16 + (b as u16 % range)) as u8;
        }
    }
    // Fallback (slight bias, but rare fallback case)
    (min_value as u16 + (bytes[31] as u16 % range)) as u8
}

/// Generates a random `u32` value from a 32-byte random seed.
pub fn random_u32(bytes: &[u8; 32]) -> u32 {
    u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]])
}

/// Generates a random `i32` value from a 32-byte random seed.
pub fn random_i32(bytes: &[u8; 32]) -> i32 {
    random_u32(bytes) as i32
}

/// Generates a random `u64` value from a 32-byte random seed.
pub fn random_u64(bytes: &[u8; 32]) -> u64 {
    u64::from_le_bytes([
        bytes[0], bytes[4], bytes[8], bytes[12], bytes[16], bytes[20], bytes[24], bytes[28],
    ])
}

/// Generates a random `i64` value from a 32-byte random seed.
pub fn random_i64(bytes: &[u8; 32]) -> i64 {
    random_u64(bytes) as i64
}

/// Generates a random boolean value from a 32-byte random seed.
pub fn random_bool(bytes: &[u8; 32]) -> bool {
    (bytes[31] % 2) == 0
}

#[cfg(test)]
mod tests {
    use ephemeral_vrf_sdk::rnd as sdk;

    use super::*;

    fn seed(salt: u8) -> [u8; 32] {
        let mut s = [0u8; 32];
        for (i, b) in s.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(7).wrapping_add(salt);
        }
        s
    }

    #[test]
    fn matches_canonical_rnd() {
        for salt in [0u8, 1, 3, 17, 200, 255] {
            let s = seed(salt);
            assert_eq!(random_u8(&s), sdk::random_u8(&s));
            assert_eq!(random_u32(&s), sdk::random_u32(&s));
            assert_eq!(random_i32(&s), sdk::random_i32(&s));
            assert_eq!(random_u64(&s), sdk::random_u64(&s));
            assert_eq!(random_i64(&s), sdk::random_i64(&s));
            assert_eq!(random_bool(&s), sdk::random_bool(&s));

            // Note: (0, 255) is intentionally excluded — `max - min + 1` overflows
            // u8 in both this port and the canonical SDK (use `random_u8` for the
            // full range instead).
            for (min, max) in [(0u8, 1u8), (1, 6), (10, 20), (100, 200), (5, 5)] {
                assert_eq!(
                    random_u8_with_range(&s, min, max),
                    sdk::random_u8_with_range(&s, min, max),
                    "range ({min},{max}) salt {salt}"
                );
            }
        }
    }

    #[test]
    fn random_u8_with_range_stays_in_bounds() {
        for salt in 0u8..32 {
            let s = seed(salt);
            for (min, max) in [(0u8, 1u8), (3, 9), (50, 60), (0, 100)] {
                let v = random_u8_with_range(&s, min, max);
                assert!(v >= min && v <= max);
            }
        }
    }
}
