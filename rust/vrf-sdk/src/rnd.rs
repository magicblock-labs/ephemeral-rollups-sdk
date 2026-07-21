/// Generates a random u8 value from a 32-byte random seed
///
/// # Arguments
///
/// * `bytes` - A 32-byte array containing random data from the VRF
///
/// # Returns
///
/// A random u8 value derived from the input bytes
pub fn random_u8(bytes: &[u8; 32]) -> u8 {
    bytes[30]
}

/// Generates a random u8 value within a specified range from a 32-byte random seed
///
/// # Arguments
///
/// * `bytes` - A 32-byte array containing random data from the VRF
/// * `min_value` - The minimum value (inclusive) of the desired range
/// * `max_value` - The maximum value (inclusive) of the desired range
///
/// # Returns
///
/// A random u8 value uniformly distributed in the range [min_value, max_value]
///
/// # Algorithm
///
/// To avoid modulo bias, the function scans through the input bytes looking for
/// a value that falls within an evenly divisible range. If no such value is found,
/// it falls back to a slightly biased approach using the last byte.
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

/// Generates a random u32 value from a 32-byte random seed
///
/// # Arguments
///
/// * `bytes` - A 32-byte array containing random data from the VRF
///
/// # Returns
///
/// A random u32 value derived from the input bytes
pub fn random_u32(bytes: &[u8; 32]) -> u32 {
    u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]])
}

/// Generates a random i32 value from a 32-byte random seed
///
/// # Arguments
///
/// * `bytes` - A 32-byte array containing random data from the VRF
///
/// # Returns
///
/// A random i32 value derived from the input bytes
pub fn random_i32(bytes: &[u8; 32]) -> i32 {
    random_u32(bytes) as i32
}

/// Generates a random u64 value from a 32-byte random seed
///
/// # Arguments
///
/// * `bytes` - A 32-byte array containing random data from the VRF
///
/// # Returns
///
/// A random u64 value derived from the input bytes
pub fn random_u64(bytes: &[u8; 32]) -> u64 {
    u64::from_le_bytes([
        bytes[0], bytes[4], bytes[8], bytes[12], bytes[16], bytes[20], bytes[24], bytes[28],
    ])
}

/// Generates a random i64 value from a 32-byte random seed
///
/// # Arguments
///
/// * `bytes` - A 32-byte array containing random data from the VRF
///
/// # Returns
///
/// A random i64 value derived from the input bytes
pub fn random_i64(bytes: &[u8; 32]) -> i64 {
    random_u64(bytes) as i64
}

/// Generates a random boolean value from a 32-byte random seed
///
/// # Arguments
///
/// * `bytes` - A 32-byte array containing random data from the VRF
///
/// # Returns
///
/// A random boolean value (true or false) derived from the input bytes
#[allow(clippy::manual_is_multiple_of)]
pub fn random_bool(bytes: &[u8; 32]) -> bool {
    (bytes[31] % 2) == 0
}

#[cfg(test)]
mod tests {
    use super::random_u8_with_range;

    #[test]
    fn random_u8_with_range_allows_equal_bounds() {
        let bytes = [42; 32];

        assert_eq!(random_u8_with_range(&bytes, 7, 7), 7);
    }

    #[test]
    fn random_u8_with_range_allows_full_u8_range() {
        let bytes = [255; 32];

        assert_eq!(random_u8_with_range(&bytes, 0, 255), 255);
    }

    #[test]
    fn random_u8_with_range_stays_within_requested_bounds() {
        let bytes = [0; 32];

        let value = random_u8_with_range(&bytes, 1, 6);

        assert!((1..=6).contains(&value));
    }
}
