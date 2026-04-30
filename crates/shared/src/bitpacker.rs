use crate::common::{
    errors::{BitPackerError, BitPackerResult},
    utils::{char6, ord6},
};
use getset::Getters;
use num_traits::PrimInt;
use std::ops::{Add, Index};

#[derive(Debug, Clone, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct BitPacker {
    bits: Vec<u8>,
    bits_len: usize,
}

impl Index<usize> for BitPacker {
    type Output = u8;

    fn index(&self, idx: usize) -> &Self::Output {
        let byte_idx: usize = self.bits.len() - 1 - idx.div_euclid(8);
        let bit_idx: usize = (idx % 8) as usize;

        let bytes: u8 = self.bits[byte_idx].clone();
        let bit: u8 = bytes >> bit_idx & 1;

        if bit == 1 { &1 } else { &0 }
    }
}

impl Add for BitPacker {
    type Output = BitPacker;

    fn add(self, rhs: Self) -> Self::Output {
        let bits_len: usize = self.bits_len + rhs.bits_len;
        let bytes_len = bits_len.div_ceil(8);

        let mut bits: Vec<u8> = vec![0u8; bytes_len];

        for i in 0..rhs.bits_len {
            let byte_idx: usize = bytes_len - 1 - i.div_euclid(8);
            let bit_idx: usize = i % 8;

            if rhs[i] == 1 {
                bits[byte_idx] |= 1 << bit_idx;
            }
        }

        for i in 0..self.bits_len {
            let byte_idx: usize = bytes_len - 1 - (i + rhs.bits_len).div_euclid(8);
            let bit_idx: usize = (i + rhs.bits_len) % 8;

            if self[i] == 1 {
                bits[byte_idx] |= 1 << bit_idx;
            }
        }

        BitPacker::from_slice(&bits, Some(bits_len))
    }
}

impl BitPacker {
    fn write_bit(&mut self, idx: usize, val: u8) -> BitPackerResult<()> {
        if idx >= self.bits_len {
            return Err(BitPackerError::IndexOutOfBounds);
        }

        let byte_idx: usize = self.bits.len() - 1 - idx.div_euclid(8) as usize;
        let bit_idx: usize = (idx % 8) as usize;

        if (self[idx] == 1 && val == 0) || (self[idx] == 0 && val == 1) {
            self.bits[byte_idx] ^= 1 << bit_idx;
        }

        Ok(())
    }

    fn write_bits<T2: PrimInt>(
        &mut self,
        val: T2,
        start_idx: Option<usize>,
    ) -> BitPackerResult<()> {
        if val != T2::zero() {
            let start_idx: usize = start_idx.unwrap_or(0);
            let end_idx: usize =
                start_idx + std::mem::size_of::<T2>() * 8 - val.leading_zeros() as usize - 1;

            for i in start_idx..=end_idx {
                let bit: T2 = (val >> (i - start_idx)) & T2::one();

                if bit == T2::one() {
                    self.write_bit(i, 1)?;
                }
            }
        }

        Ok(())
    }

    pub fn from_int<T: PrimInt>(val: T, bits_len: Option<usize>) -> Self {
        let bits_len: usize =
            bits_len.unwrap_or(std::mem::size_of::<T>() * 8 - val.leading_zeros() as usize);
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut bitpacker: BitPacker = Self {
            bits: vec![0u8; bytes_len],
            bits_len: bits_len,
        };

        bitpacker.write_bits::<T>(val, None);

        bitpacker
    }

    pub fn from_str(val: &str, bits_len: Option<usize>) -> Self {
        let bits_len: usize = bits_len.unwrap_or(6 * val.len());
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut bitpacker: BitPacker = Self {
            bits: vec![0u8; bytes_len],
            bits_len: bits_len,
        };

        for (i, c) in val.to_ascii_uppercase().chars().enumerate() {
            let ord: u8 = ord6(c);
            bitpacker.write_bits::<u8>(ord, Some(i * 6));
        }

        bitpacker
    }

    pub fn from_slice(bits: &[u8], bits_len: Option<usize>) -> Self {
        Self {
            bits: Vec::from(bits),
            bits_len: bits_len.unwrap_or(bits.len()),
        }
    }

    pub fn slice(&self, start_idx: Option<usize>, end_idx: Option<usize>) -> BitPackerResult<Self> {
        let start_i: usize = start_idx.unwrap_or(0);
        let end_i: usize = end_idx.unwrap_or(self.bits_len - 1);

        if start_i >= self.bits_len || end_i >= self.bits_len {
            return Err(BitPackerError::IndexOutOfBounds);
        }

        let bits_len: usize = end_i - start_i + 1;
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut bits: Vec<u8> = vec![0u8; bytes_len];

        for slice_i in 0..bits_len {
            let original_i: usize = slice_i + start_i;

            let slice_byte_idx: usize = bytes_len - 1 - slice_i.div_euclid(8);
            let slice_bit_idx: usize = slice_i % 8;

            if self[original_i] == 1 {
                bits[slice_byte_idx] |= 1 << slice_bit_idx;
            }
        }

        let bitpacker: BitPacker = BitPacker::from_slice(&bits, Some(bits_len));

        Ok(bitpacker)
    }

    pub fn extract_int<T: PrimInt>(
        &self,
        start_idx: Option<usize>,
        end_idx: Option<usize>,
    ) -> BitPackerResult<T> {
        let start_idx: usize = start_idx.unwrap_or(0);
        let end_idx: usize = end_idx.unwrap_or(self.bits_len - 1);

        if start_idx >= self.bits_len || end_idx >= self.bits_len {
            return Err(BitPackerError::IndexOutOfBounds);
        }

        let mut val: T = T::zero();

        let int_slice: BitPacker = self.slice(Some(start_idx), Some(end_idx))?;

        for i in 0..int_slice.bits_len {
            if int_slice[i] == 1 {
                val = val | (T::one() << i);
            }
        }

        Ok(val)
    }

    pub fn extract_str(
        &self,
        start_idx: Option<usize>,
        end_idx: Option<usize>,
    ) -> BitPackerResult<String> {
        let start_idx: usize = start_idx.unwrap_or(0);
        let end_idx: usize = end_idx.unwrap_or(self.bits_len - 1);

        if start_idx >= self.bits_len || end_idx >= self.bits_len {
            return Err(BitPackerError::IndexOutOfBounds);
        }

        let bits_len: usize = end_idx - start_idx + 1;
        let bits_len: usize = bits_len - bits_len % 6;

        let str_slice: BitPacker = self.slice(Some(start_idx), Some(end_idx))?;

        let mut extracted_str: String = String::new();

        let mut chr_ord: u8 = 0;

        for i in 0..bits_len {
            if str_slice[i] == 1 {
                chr_ord |= 1 << i % 6;
            }

            if i % 6 == 5 {
                if chr_ord != 0 {
                    extracted_str += &char6(chr_ord).to_string();
                    chr_ord = 0;
                }
            }
        }

        Ok(extracted_str)
    }

    pub fn to_bin_str(&self) -> String {
        let mut bin_str: String = String::new();

        for i in 0..self.bits_len {
            if self[i] == 1 {
                bin_str = "1".to_string() + &bin_str;
            } else {
                bin_str = "0".to_string() + &bin_str;
            }
        }

        bin_str
    }
}
