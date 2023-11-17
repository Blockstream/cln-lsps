use anyhow::Context;
/// Contains LSPS0 related utilities
///
/// To determine if a Lightning-node is an LSP-server you can
/// inspect the feature flags.
///
use std::str::FromStr;

pub const LSP_SERVER_FEATURE_BIT: usize = 729;

pub struct FeatureBitMap(Vec<u8>);

impl FromStr for FeatureBitMap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = hex::decode(s).with_context(|| format!("Failed to parse feature-bitmap"))?;
        return Ok(FeatureBitMap(hex));
    }
}

impl AsRef<[u8]> for &FeatureBitMap {
    fn as_ref(&self) -> &[u8] {
        return &self.0;
    }
}

impl FeatureBitMap {
    pub fn new(bitmap: Vec<u8>) -> Self {
        return FeatureBitMap(bitmap);
    }
}

/// Returns True if the feature-bit at index usize is enabled
pub fn is_feature_bit_enabled<T: AsRef<[u8]>>(bitmap: T, index: usize) -> bool {
    let bm = bitmap.as_ref();
    let n_bytes = bm.len();
    let (byte_index, bit_index) = (index / 8, index % 8);

    // The index doesn't fit in the byte-array
    if byte_index >= n_bytes {
        return false;
    }

    let selected_byte = bm[n_bytes - 1 - byte_index];
    let bit_mask = 1u8 << (bit_index);

    return (selected_byte & bit_mask) != 0;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_bitmap() {
        // Check the lowest bits in the bitmap
        let feature_bitmap_00 = FeatureBitMap::from_str(&"01").unwrap();
        let feature_bitmap_01 = FeatureBitMap::from_str(&"02").unwrap();
        let feature_bitmap_02 = FeatureBitMap::from_str(&"04").unwrap();
        let feature_bitmap_03 = FeatureBitMap::from_str(&"08").unwrap();
        let feature_bitmap_04 = FeatureBitMap::from_str(&"10").unwrap();
        let feature_bitmap_05 = FeatureBitMap::from_str(&"20").unwrap();
        let feature_bitmap_06 = FeatureBitMap::from_str(&"40").unwrap();
        let feature_bitmap_07 = FeatureBitMap::from_str(&"80").unwrap();

        // Check that the expected bit is enabled
        assert!(is_feature_bit_enabled(&feature_bitmap_00, 0));
        assert!(is_feature_bit_enabled(&feature_bitmap_01, 1));
        assert!(is_feature_bit_enabled(&feature_bitmap_02, 2));
        assert!(is_feature_bit_enabled(&feature_bitmap_03, 3));
        assert!(is_feature_bit_enabled(&feature_bitmap_04, 4));
        assert!(is_feature_bit_enabled(&feature_bitmap_05, 5));
        assert!(is_feature_bit_enabled(&feature_bitmap_06, 6));
        assert!(is_feature_bit_enabled(&feature_bitmap_07, 7));

        // Check that other bits are disabled
        assert!(!is_feature_bit_enabled(&feature_bitmap_01, 0));
        assert!(!is_feature_bit_enabled(&feature_bitmap_01, 2));
        assert!(!is_feature_bit_enabled(&feature_bitmap_01, 3));
        assert!(!is_feature_bit_enabled(&feature_bitmap_01, 4));
        assert!(!is_feature_bit_enabled(&feature_bitmap_01, 5));
        assert!(!is_feature_bit_enabled(&feature_bitmap_01, 6));
        assert!(!is_feature_bit_enabled(&feature_bitmap_01, 7));
        assert!(!is_feature_bit_enabled(&feature_bitmap_01, 8));
        assert!(!is_feature_bit_enabled(&feature_bitmap_01, 9));

        assert!(!is_feature_bit_enabled(&feature_bitmap_01, 1000));
    }

    #[test]
    fn test_lsps_option_enabled_bitmap() {
        // Copied from LSPS0
        // This set bit number 729
        let data = "0200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let bitmap = FeatureBitMap::from_str(&data).unwrap();

        // Check that the expected bit is enabled
        assert!(is_feature_bit_enabled(&bitmap, LSP_SERVER_FEATURE_BIT));

        // Check that the expected bit is disabled
        assert!(!is_feature_bit_enabled(&bitmap, 728));
        assert!(!is_feature_bit_enabled(&bitmap, 730));
    }
}
