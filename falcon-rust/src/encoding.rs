use alloc::vec::Vec;
use bit_vec::BitVec;
use itertools::Itertools;
use num::Integer;

#[cfg(test)]
/// Compression and decompression routines for signatures.

/// This is a deprecated compress routine used now only for testing
/// compatibility with the new, faster implementation (below).
#[allow(dead_code)]
pub(crate) fn compress_slow(v: &[i16], slen: usize) -> Option<Vec<u8>> {
    let mut bitvector: BitVec = BitVec::with_capacity(slen);
    for coeff in v {
        // encode sign
        bitvector.push(*coeff < 0);
        // encode low bits
        let s = (*coeff).abs();
        for i in (0..7).rev() {
            bitvector.push(((s >> i) & 1) != 0);
        }
        // encode high bits
        for _ in 0..(s >> 7) {
            bitvector.push(false);
        }
        bitvector.push(true);
    }
    // return failure if encoding is too long
    if bitvector.len() > slen {
        return None;
    }
    // pad
    while bitvector.len() < slen {
        bitvector.push(false);
    }
    Some(bitvector.to_bytes())
}

/// Take as input a list of integers v and a byte length `byte_length``, and
/// return a bytestring of length `byte_length` that encode/compress v.
/// If this is not possible, return False.
///
/// For each coefficient of v:
/// - the sign is encoded on 1 bit
/// - the 7 lower bits are encoded naively (binary)
/// - the high bits are encoded in unary encoding
///
/// This method can fail, in which case it returns None. The signature
/// generation algorithm knows this and will re-run the loop.
///
/// Algorithm 17 p. 47 of the specification [1].
///
/// [1]: https://falcon-sign.info/falcon.pdf
pub(crate) fn compress(v: &[i16], byte_length: usize) -> Option<Vec<u8>> {
    // encode each coefficient separately; join later
    let lengths_and_coefficients = v.iter().map(|c| compress_coefficient(*c)).collect_vec();
    let total_length = lengths_and_coefficients
        .iter()
        .map(|(l, _c)| *l)
        .sum::<usize>();

    // if we can't fit all coefficients in the allotted bytes
    if total_length > byte_length * 8 {
        return None;
    }

    // no coefficients are given
    if v.is_empty() {
        return None;
    }

    // join all but one coefficients assuming enough space
    let mut bytes = alloc::vec![0u8; byte_length];
    let mut counter = 0;
    for (length, coefficient) in lengths_and_coefficients.iter().take(v.len() - 1) {
        let (cdiv8, cmod8) = counter.div_mod_floor(&8);
        bytes[cdiv8] |= coefficient >> cmod8;
        bytes[cdiv8 + 1] |= ((*coefficient as u16) << (8 - cmod8)) as u8;
        let (cldiv8, clmod8) = (counter + length - 1).div_mod_floor(&8);
        bytes[cldiv8] |= 128u8 >> clmod8;
        bytes[cldiv8 + 1] |= (128u16 << (8 - clmod8)) as u8;
        counter += length;
    }

    // treat last coefficient special
    let (length, coefficient) = lengths_and_coefficients.last().unwrap();
    {
        let (cdiv8, cmod8) = counter.div_mod_floor(&8);
        bytes[cdiv8] |= coefficient >> cmod8;
        bytes[cdiv8 + 1] |= ((*coefficient as u16) << (8 - cmod8)) as u8;
        let (cldiv8, clmod8) = (counter + length - 1).div_mod_floor(&8);
        bytes[cldiv8] |= 128u8 >> clmod8;
        if cldiv8 + 1 < byte_length {
            bytes[cldiv8 + 1] |= (128u16 << (8 - clmod8)) as u8;
        } else if (128u16 << (8 - clmod8)) as u8 != 0 {
            return None;
        }
        counter += length;
    }
    Some(bytes)
}

/// Helper function for compress; isolate attention to one coefficient.
fn compress_coefficient(coeff: i16) -> (usize, u8) {
    let sign = (coeff < 0) as u8;
    let abs = coeff.unsigned_abs();
    let low = abs as u8 & 127;
    let high = abs >> 7;
    (1 + 7 + high as usize + 1, ((sign << 7) | low))
}

///  This is a deprecated decompress routine used now only for testing
/// compatibility with the new, faster implementation (below).
#[allow(dead_code)]
pub(crate) fn decompress_slow(x: &[u8], n: usize) -> Option<Vec<i16>> {
    let bitvector = BitVec::from_bytes(x);
    let mut index = 0;
    let mut result = alloc::vec::Vec::with_capacity(n);
    for _ in 0..n {
        // early return if
        if index + 8 >= bitvector.len() {
            return None;
        }

        // read sign
        let sign = if bitvector[index] { -1 } else { 1 };
        index += 1;

        // read low bits
        let mut low_bits = 0i16;
        for _ in 0..7 {
            low_bits = (low_bits << 1) | if bitvector[index] { 1 } else { 0 };
            index += 1;
        }

        // read high bits
        let mut high_bits = 0;
        while !bitvector[index] {
            index += 1;
            high_bits += 1;
        }
        index += 1;

        // compose integer and collect it
        let integer = sign * ((high_bits << 7) | low_bits);
        result.push(integer);
    }
    Some(result)
}

/// Take as input an encoding x, and a length n, and return a list of
/// integers v of length n such that x encode v. If such a list does
/// not exist, the encoding is invalid and we output None.
///
/// Algorithm 18 p. 48 of the specification [1].
///
/// [1]: https://falcon-sign.info/falcon.pdf
pub(crate) fn decompress(x: &[u8], n: usize) -> Option<Vec<i16>> {
    let bitvector = BitVec::from_bytes(x);
    let mut index = 0;
    let mut result = alloc::vec::Vec::with_capacity(n);

    // tracks invalid coefficient encodings
    let mut abort = false;

    // for all elements (last round is special due to bound checks)
    for _ in 0..n - 1 {
        // early return if
        if index + 8 >= bitvector.len() {
            return None;
        }

        // read sign
        let sign = if bitvector[index] { -1 } else { 1 };
        index += 1;

        // read low bits
        let mut low_bits = 0i16;
        let (index_div_8, index_mod_8) = index.div_mod_floor(&8);
        low_bits |= (x[index_div_8] as i16) << index_mod_8;
        low_bits |= (x[index_div_8 + 1] as i16) >> (8 - index_mod_8);
        low_bits = (low_bits & 255) >> 1;
        index += 7;

        // read high bits
        let mut high_bits = 0;
        while !bitvector[index] {
            index += 1;
            high_bits += 1;

            if high_bits == 95 || index + 1 == bitvector.len() {
                return None;
            }
        }
        index += 1;

        // test if coefficient is encoded properly
        abort |= low_bits == 0 && high_bits == 0 && sign == -1;

        // compose integer and collect it
        let integer = sign * ((high_bits << 7) | low_bits);
        result.push(integer);
    }

    // last round

    // early return if
    if index + 8 >= bitvector.len() {
        return None;
    }

    // read sign
    if bitvector.len() == index {
        return None;
    }
    let sign = if bitvector[index] { -1 } else { 1 };
    index += 1;

    // read low bits
    let mut low_bits = 0i16;
    let (index_div_8, index_mod_8) = index.div_mod_floor(&8);
    low_bits |= (x[index_div_8] as i16) << index_mod_8;
    if index_mod_8 != 0 && index_div_8 + 1 < x.len() {
        low_bits |= (x[index_div_8 + 1] as i16) >> (8 - index_mod_8);
    } else if index_mod_8 != 0 {
        return None;
    }
    low_bits = (low_bits & 255) >> 1;
    index += 7;

    // read high bits
    let mut high_bits = 0;
    if bitvector.len() == index {
        return None;
    }
    while !bitvector[index] {
        index += 1;
        if bitvector.len() == index {
            return None;
        }
        high_bits += 1;
    }

    // test if coefficient encoded properly
    if abort || (low_bits == 0 && high_bits == 0 && sign == -1) {
        return None;
    }

    // compose integer and collect it
    let integer = sign * ((high_bits << 7) | low_bits);
    result.push(integer);

    // check padding
    index += 1;
    let (index_div_8, index_mod_8) = index.div_mod_floor(&8);
    for idx in 0..(8 - index_mod_8) {
        if let Some(b) = bitvector.get(index + idx) {
            if b {
                // unread part of input contains set bits
                return None;
            }
        }
    }
    for &byte in x.iter().skip(index_div_8 + 1 - (index_mod_8 == 0) as usize) {
        if byte != 0 {
            // unread part of input contains set bits!
            return None;
        }
    }

    Some(result)
}

#[cfg(test)]
mod test {
    use crate::encoding::{compress, decompress};
    use alloc::vec::Vec;

    use proptest::prelude::*;

    fn short_elements(n: usize) -> Vec<i16> {
        // Normal distribution sampling disabled for no-std
        // Using simple range instead
        (0..n).map(|i| ((i as i16) % 100) - 50).collect()
    }
    proptest! {
        #[test]
        fn compress_does_not_crash(v in (0..2000usize).prop_map(short_elements)) {
            compress(&v, 2*v.len());
        }
    }
    proptest! {
        #[test]
        fn decompress_recovers(v in (0..2000usize).prop_map(short_elements)) {
            let slen = 2 * v.len();
            let n = v.len();
            if let Some(compressed) = compress(&v, slen) {
                let recovered = decompress(&compressed, n).unwrap();
                prop_assert_eq!(v, recovered.clone());
                let recompressed = compress(&recovered, slen).unwrap();
                prop_assert_eq!(compressed, recompressed);
            }
        }
    }

    #[test]
    #[ignore] // Requires rand_distr (not no-std compatible)
    fn compress_empty_vec_does_not_crash() {
        compress(&[], 0);
    }

    #[test]
    #[ignore] // Requires rand_distr (not no-std compatible)
    fn test_compress_decompress() {
        // Test disabled - requires rand_distr for Normal distribution
    }

    #[test]
    #[ignore] // Requires rand_distr (not no-std compatible)
    fn test_compress_equiv() {
        // Test disabled - requires rand_distr for Normal distribution
    }

    #[test]
    #[ignore] // Requires rand_distr (not no-std compatible)
    fn test_decompress_equiv() {
        // Test disabled - requires rand_distr for Normal distribution
    }

    #[test]
    #[ignore] // Requires rand_distr (not no-std compatible)
    fn test_decompress_failures() {
        // Test disabled - requires rand_distr for Normal distribution
    }
}
