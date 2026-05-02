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
        let byte_idx: usize = self.bits.len() - 1 - (idx >> 3);
        let bit_idx: usize = idx & 7;

        if (self.bits[byte_idx] >> bit_idx) & 1 == 1 {
            &1
        } else {
            &0
        }
    }
}

impl Add for BitPacker {
    type Output = BitPacker;

    fn add(self, rhs: Self) -> Self::Output {
        let bits_len: usize = self.bits_len + rhs.bits_len;
        let bytes_len: usize = bits_len.div_ceil(8);
        let mut bits: Vec<u8> = vec![0u8; bytes_len];

        let mut write = |idx: usize, bit: u8| {
            if bit == 1 {
                let byte_idx = bytes_len - 1 - (idx >> 3);
                bits[byte_idx] |= 1 << (idx & 7);
            }
        };

        for i in 0..rhs.bits_len {
            write(i, rhs[i]);
        }

        for i in 0..self.bits_len {
            write(i + rhs.bits_len, self.get_bit_unchecked(i));
        }

        BitPacker::from_slice(&bits, Some(bits_len))
    }
}

impl BitPacker {
    #[inline]
    fn get_bit_unchecked(&self, idx: usize) -> u8 {
        let byte_idx: usize = self.bits.len() - 1 - (idx >> 3);
        (self.bits[byte_idx] >> (idx & 7)) & 1
    }

    fn write_bit(&mut self, idx: usize, val: u8) -> BitPackerResult<()> {
        if idx >= self.bits_len {
            return Err(BitPackerError::IndexOutOfBounds);
        }

        let byte_idx: usize = self.bits.len() - 1 - (idx >> 3);
        let bit_idx: usize = idx & 7;

        if val == 1 {
            self.bits[byte_idx] |= 1 << bit_idx;
        } else {
            self.bits[byte_idx] &= !(1 << bit_idx);
        }

        Ok(())
    }

    fn write_bits<T2: PrimInt>(
        &mut self,
        val: T2,
        start_idx: Option<usize>,
    ) -> BitPackerResult<()> {
        if val == T2::zero() {
            return Ok(());
        }

        let start: usize = start_idx.unwrap_or(0);
        let bits_to_write: usize = std::mem::size_of::<T2>() * 8 - val.leading_zeros() as usize;

        for i in 0..bits_to_write {
            if ((val >> i) & T2::one()) == T2::one() {
                self.write_bit(start + i, 1)?;
            }
        }
        Ok(())
    }

    pub fn from_int<T: PrimInt>(val: T, bits_len: Option<usize>) -> Self {
        let bits_len: usize =
            bits_len.unwrap_or_else(|| std::mem::size_of::<T>() * 8 - val.leading_zeros() as usize);
        let mut bitpacker = Self {
            bits: vec![0u8; bits_len.div_ceil(8)],
            bits_len,
        };
        let _ = bitpacker.write_bits(val, None);
        bitpacker
    }

    pub fn from_str(val: &str, bits_len: Option<usize>) -> Self {
        let bits_len: usize = bits_len.unwrap_or(6 * val.len());
        let mut bitpacker = Self {
            bits: vec![0u8; bits_len.div_ceil(8)],
            bits_len,
        };

        for (i, c) in val.chars().enumerate() {
            let ord: u8 = ord6(c.to_ascii_uppercase());
            let _ = bitpacker.write_bits(ord, Some(i * 6));
        }
        bitpacker
    }

    pub fn from_slice(bits: &[u8], bits_len: Option<usize>) -> Self {
        Self {
            bits: bits.to_vec(),
            bits_len: bits_len.unwrap_or(bits.len() * 8),
        }
    }

    pub fn slice(&self, start_idx: Option<usize>, end_idx: Option<usize>) -> BitPackerResult<Self> {
        let start: usize = start_idx.unwrap_or(0);
        let end: usize = end_idx.unwrap_or(self.bits_len.saturating_sub(1));

        if start >= self.bits_len || end >= self.bits_len {
            return Err(BitPackerError::IndexOutOfBounds);
        }

        let slice_len: usize = end - start + 1;
        let bytes_len: usize = slice_len.div_ceil(8);
        let mut bits: Vec<u8> = vec![0u8; bytes_len];

        for i in 0..slice_len {
            if self.get_bit_unchecked(start + i) == 1 {
                bits[bytes_len - 1 - (i >> 3)] |= 1 << (i & 7);
            }
        }

        Ok(BitPacker::from_slice(&bits, Some(slice_len)))
    }

    pub fn extract_int<T: PrimInt>(
        &self,
        start_idx: Option<usize>,
        end_idx: Option<usize>,
    ) -> BitPackerResult<T> {
        let start: usize = start_idx.unwrap_or(0);
        let end: usize = end_idx.unwrap_or(self.bits_len.saturating_sub(1));

        if start >= self.bits_len || end >= self.bits_len {
            return Err(BitPackerError::IndexOutOfBounds);
        }

        let mut val = T::zero();
        for i in 0..=(end - start) {
            if self.get_bit_unchecked(start + i) == 1 {
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
        let start: usize = start_idx.unwrap_or(0);
        let end: usize = end_idx.unwrap_or(self.bits_len.saturating_sub(1));

        if start >= self.bits_len || end >= self.bits_len {
            return Err(BitPackerError::IndexOutOfBounds);
        }

        let len: usize = end - start + 1;
        let num_chars: usize = len / 6;

        let mut extracted_str: String = String::with_capacity(num_chars);

        for char_idx in 0..num_chars {
            let mut chr_ord: u8 = 0;
            let char_start = start + char_idx * 6;

            for bit_idx in 0..6 {
                if self.get_bit_unchecked(char_start + bit_idx) == 1 {
                    chr_ord |= 1 << bit_idx;
                }
            }

            if chr_ord != 0 {
                extracted_str.push(char6(chr_ord));
            }
        }

        Ok(extracted_str)
    }

    pub fn to_bin_str(&self) -> String {
        let mut bin_str = String::with_capacity(self.bits_len);
        for i in (0..self.bits_len).rev() {
            bin_str.push(if self.get_bit_unchecked(i) == 1 {
                '1'
            } else {
                '0'
            });
        }
        bin_str
    }
}
