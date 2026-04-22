use std::ops::{Add, Index};

use num_traits::PrimInt;

use crate::common::{
    types::{BitPackerError, BitPackerResult},
    utils::{char6, ord6},
};

#[derive(Debug, Clone, PartialEq)]
pub struct BitPacker {
    bits: Vec<u8>,
    pub bits_len: usize,
}

impl Index<usize> for BitPacker {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        let byte_idx: usize = self.bits.len() - 1 - index.div_euclid(8);
        let bit_idx: usize = (index % 8) as usize;

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
    fn write_bit(&mut self, index: usize, value: u8) -> BitPackerResult<()> {
        if index >= self.bits_len {
            return Err(BitPackerError::IndexOutOfBounds);
        }

        let byte_idx: usize = self.bits.len() - 1 - index.div_euclid(8) as usize;
        let bit_idx: usize = (index % 8) as usize;

        if (self[index] == 1 && value == 0) || (self[index] == 0 && value == 1) {
            self.bits[byte_idx] ^= 1 << bit_idx;
        }

        Ok(())
    }

    fn write_bits<T2: PrimInt>(
        &mut self,
        value: T2,
        start_i: Option<usize>,
    ) -> BitPackerResult<()> {
        if value != T2::zero() {
            let start_i: usize = start_i.unwrap_or(0);
            let end_i: usize =
                start_i + std::mem::size_of::<T2>() * 8 - value.leading_zeros() as usize - 1;

            for i in start_i..=end_i {
                let bit: T2 = (value >> (i - start_i)) & T2::one();

                if bit == T2::one() {
                    self.write_bit(i, 1)?;
                }
            }
        }

        Ok(())
    }

    pub fn bits(&self) -> &[u8] {
        &self.bits
    }

    pub fn from_int<T: PrimInt>(value: T, bits_len: Option<usize>) -> Self {
        let bits_len: usize =
            bits_len.unwrap_or(std::mem::size_of::<T>() * 8 - value.leading_zeros() as usize);
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut bitpacker: BitPacker = Self {
            bits: vec![0u8; bytes_len],
            bits_len: bits_len,
        };

        bitpacker.write_bits::<T>(value, None);

        bitpacker
    }

    pub fn from_str(value: &str, bits_len: Option<usize>) -> Self {
        let bits_len: usize = bits_len.unwrap_or(6 * value.len());
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut bitpacker: BitPacker = Self {
            bits: vec![0u8; bytes_len],
            bits_len: bits_len,
        };

        for (i, c) in value.to_ascii_uppercase().chars().enumerate() {
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

    pub fn slice(&self, start_i: Option<usize>, end_i: Option<usize>) -> BitPackerResult<Self> {
        let start_i: usize = start_i.unwrap_or(0);
        let end_i: usize = end_i.unwrap_or(self.bits_len - 1);

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
        start_i: Option<usize>,
        end_i: Option<usize>,
    ) -> BitPackerResult<T> {
        let start_i: usize = start_i.unwrap_or(0);
        let end_i: usize = end_i.unwrap_or(self.bits_len - 1);

        if start_i >= self.bits_len || end_i >= self.bits_len {
            return Err(BitPackerError::IndexOutOfBounds);
        }

        let mut value: T = T::zero();

        let int_slice: BitPacker = self.slice(Some(start_i), Some(end_i))?;

        for i in 0..int_slice.bits_len {
            if int_slice[i] == 1 {
                value = value | (T::one() << i);
            }
        }

        Ok(value)
    }

    pub fn extract_str(
        &self,
        start_i: Option<usize>,
        end_i: Option<usize>,
    ) -> BitPackerResult<String> {
        let start_i: usize = start_i.unwrap_or(0);
        let end_i: usize = end_i.unwrap_or(self.bits_len - 1);

        if start_i >= self.bits_len || end_i >= self.bits_len {
            return Err(BitPackerError::IndexOutOfBounds);
        }

        let bits_len: usize = end_i - start_i + 1;
        let bits_len: usize = bits_len - bits_len % 6;

        let str_slice: BitPacker = self.slice(Some(start_i), Some(end_i))?;

        let mut extracted_str: String = String::new();

        let mut char_ord: u8 = 0;

        for i in 0..bits_len {
            if str_slice[i] == 1 {
                char_ord |= 1 << i % 6;
            }

            if i % 6 == 5 {
                if char_ord != 0 {
                    extracted_str += &char6(char_ord).to_string();
                    char_ord = 0;
                }
            }
        }

        Ok(extracted_str)
    }

    pub fn to_bin_string(&self) -> String {
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
