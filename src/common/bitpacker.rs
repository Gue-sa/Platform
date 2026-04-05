use std::{cmp::min, ops::Index};

use num_traits::PrimInt;

use crate::common::utils;


#[derive(Debug)]
pub struct BitPacker<T> {
    value: T,
    bits: Vec<u8>,
    bits_len: usize,
    bytes_len: usize
}


impl<T> Index<u8> for BitPacker<T> {
    type Output = u8;

    fn index(&self, index: u8) -> &Self::Output {
        let byte_idx: usize = self.bytes_len - 1 - index.div_euclid(8) as usize;
        let bit_idx: usize = (index % 8) as usize;

        let bytes: u8 = self.bits[byte_idx].clone();
        let bit: u8 = bytes >> bit_idx & 1;
        
        if bit == 1 {
            &1
        } else {
            &0
        }
    }
}


impl<T> BitPacker<T> {
    fn write_bit(&mut self, index: usize, value: u8) -> Result<(), &'static str> {
        if value != 0 && value != 1 {
            Err("Valeur de bit illégale.")
        } else {
            let byte_idx: usize = self.bytes_len - 1 - index.div_euclid(8) as usize;
            let bit_idx: usize = (index % 8) as usize;

            if (self[index as u8] == 1 && value == 0) || (self[index as u8] == 0 && value == 1) {
                self.bits[byte_idx] ^= 1 << bit_idx;
            }

            Ok(())
        }
    }


    fn write_bits<T2: PrimInt>(&mut self, value: T2, start_i: Option<usize>) -> Result<(), &'static str> {
        if value != T2::zero() {
            let start_i: usize = start_i.unwrap_or(0);
            let end_i: usize = start_i + std::mem::size_of::<T2>() * 8 - value.leading_zeros() as usize - 1;

            for i in start_i..=end_i {
                let bit: T2 = (value >> (i - start_i)) & T2::one();

                if bit == T2::one() {
                    self.write_bit(i, 1)?;
                }
            }
        }

        Ok(())
    }


    pub fn bits(&self) -> Box<[u8]> {
        self.bits.clone().into_boxed_slice()
    }
}


impl<T: PrimInt> BitPacker<T> {
    pub fn from_int(value: T, bits_len: Option<usize>) -> Result<Self, &'static str> {
        let bits_len: usize = bits_len.unwrap_or(std::mem::size_of::<T>() * 8 - value.leading_zeros() as usize);
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut bitpacker: BitPacker<T> = Self {
            value: value,
            bits: vec![0u8; bytes_len],
            bits_len: bits_len,
            bytes_len: bytes_len
        };

        let _ = bitpacker.write_bits::<T>(value, None)?;

        Ok(bitpacker)
    }


    pub fn from_int_box_slice<T2: PrimInt>(bits: Box<[u8]>, start_i: Option<usize>, end_i: Option<usize>) -> Result<Self, &'static str> {
        let start_i: usize = start_i.unwrap_or(0);
        let end_i: usize = end_i.unwrap_or(bits.len() * 8);

        let bits_len: usize = end_i - start_i + 1;
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut new_bits: Vec<u8> = vec![0u8; bytes_len];

        for i in 0..bits_len {
            let byte_idx: usize = bits.len() - 1 - (i + start_i).div_euclid(8);
            let bit_idx: usize = (i + start_i) % 8;

            let new_byte_idx: usize = bytes_len - 1 - i.div_euclid(8);
            let new_bit_idx: usize = i % 8;

            let bit: u8 = (bits[byte_idx] >> bit_idx) & 1;

            if bit == 1 {
                new_bits[new_byte_idx] |= 1 << new_bit_idx;
            }
        }

        let bitpacker: BitPacker<T> = BitPacker::parse_int(bits.into())?;

        Ok(bitpacker)
    }
    

    pub fn parse_int(bits: Box<[u8]>) -> Result<Self, &'static str> {
        let bits_len: usize = min(bits.len() * 8, std::mem::size_of::<T>() * 8);
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut bitpacker: BitPacker<T> = Self {
            value: T::zero(),
            bits: bits[(bits.len() - bytes_len)..].to_vec(),
            bits_len: bits_len,
            bytes_len: bytes_len
        };

        for i in 0..bits_len {
            let byte_idx: usize = bitpacker.bytes_len - 1 - i.div_euclid(8);
            let bit_idx: usize = i % 8;

            if (bits[byte_idx] >> bit_idx) & 1 == 1 {
                bitpacker.value = bitpacker.value | (T::one() << i);
            }
        };
        
        Ok(bitpacker)
    }


    pub fn slice_int(&self, start_i: Option<usize>, end_i: Option<usize>) -> Result<BitPacker<T>, &'static str> {
        let start_i: usize = start_i.unwrap_or(0);
        let end_i: usize = end_i.unwrap_or(self.bits_len - 1);

        let bits_len: usize = end_i - start_i + 1;
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut bits: Vec<u8> = vec![0u8; bytes_len];

        for slice_i in 0..bits_len {
            let original_i: usize = slice_i + start_i;

            let slice_byte_idx: usize = bytes_len - 1 - slice_i.div_euclid(8);
            let slice_bit_idx: usize = slice_i % 8;

            if self[original_i as u8] == 1 {
                bits[slice_byte_idx] |= 1 << slice_bit_idx;
            }
        }

        let mut bitpacker: BitPacker<T> = BitPacker::parse_int(bits.into_boxed_slice())?;

        bitpacker.bits_len = bits_len;

        Ok(bitpacker)
    }


    pub fn concat_int(&mut self, bit_packer: BitPacker<T>) -> Result<(), &'static str> {
        self.bits_len = self.bits_len + bit_packer.bits_len;
        self.bytes_len = self.bits_len.div_ceil(8);

        while self.bits.len() < self.bytes_len {
            self.bits.insert(0, 0);
        }

       let _ = self.write_bits(bit_packer.value, Some(self.bits_len - 1));

       self.value = BitPacker::parse_int(self.bits()).unwrap().value;

       Ok(())
    }
}


impl BitPacker<String> {
    pub fn from_str(value: &str, bits_len: Option<usize>) -> Result<Self, &'static str> {
        let bits_len: usize = 6 * value.len();
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut bitpacker: BitPacker<String> = Self {
            value: value.to_string(),
            bits: vec![0u8; bytes_len],
            bits_len: bits_len,
            bytes_len: bytes_len
        };

        for (i, c) in value.chars().enumerate() {
            let ord: u8 = utils::ord6(c);
            let _ = bitpacker.write_bits::<u8>(ord, Some(i * 6))?;
        }

        Ok(bitpacker)
    }


    pub fn from_str_box_slice<T2: PrimInt>(bits: Box<[u8]>, start_i: Option<usize>, end_i: Option<usize>) -> Result<Self, &'static str> {
        let start_i: usize = start_i.unwrap_or(0);
        let end_i: usize = end_i.unwrap_or(bits.len() * 8);

        let bits_len: usize = end_i - start_i + 1;
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut new_bits: Vec<u8> = vec![0u8; bytes_len];

        for i in 0..bits_len {
            let byte_idx: usize = bits.len() - 1 - (i + start_i).div_euclid(8);
            let bit_idx: usize = (i + start_i) % 8;

            let new_byte_idx: usize = bytes_len - 1 - i.div_euclid(8);
            let new_bit_idx: usize = i % 8;

            let bit: u8 = (bits[byte_idx] >> bit_idx) & 1;

            if bit == 1 {
                new_bits[new_byte_idx] |= 1 << new_bit_idx;
            }
        }

        let bitpacker: BitPacker<String> = BitPacker::parse_str(bits.into())?;

        Ok(bitpacker)
    }


    pub fn parse_str(bits: Box<[u8]>) -> Result<Self, &'static str> {
        let bits_len: usize = bits.len() * 8;
        let bits_len: usize = bits_len - bits_len % 6;
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut bitpacker: BitPacker<String> = Self {
            value: String::new(),
            bits: bits[(bits.len() - bytes_len)..].to_vec(),
            bits_len: bits_len,
            bytes_len: bytes_len
        };

        let mut char_ord: u8 = 0;

        for i in 0..bits_len {
            let byte_idx: usize = bitpacker.bytes_len - 1 - i.div_euclid(8);
            let bit_idx: usize = i % 8;

            if (bits[byte_idx] >> bit_idx) & 1 == 1 {
                char_ord |= 1 << i % 6;
            }

            if i % 6 == 5 {
                bitpacker.value += &utils::char6(char_ord).to_string();
                char_ord = 0;
            }
        };

        Ok(bitpacker)
    }
    
    
    pub fn slice_str(&self, start_i: Option<usize>, end_i: Option<usize>) -> Result<BitPacker<String>, &'static str> {
        let start_i: usize = start_i.unwrap_or(0);
        let end_i: usize = end_i.unwrap_or(self.bits_len - 1);

        let bits_len: usize = end_i - start_i + 1;
        let bytes_len: usize = bits_len.div_ceil(8);

        let mut bits: Vec<u8> = vec![0u8; bytes_len];

        for slice_i in 0..bits_len {
            let original_i: usize = slice_i + start_i;

            let slice_byte_idx: usize = bytes_len - 1 - slice_i.div_euclid(8);
            let slice_bit_idx: usize = slice_i % 8;

            if self[original_i as u8] == 1 {
                bits[slice_byte_idx] |= 1 << slice_bit_idx;
            }
        }

        let mut bitpacker: BitPacker<String> = BitPacker::parse_str(bits.into_boxed_slice())?;

        bitpacker.bits_len = bits_len;

        Ok(bitpacker)
    }


    pub fn concat_str(&mut self, bit_packer: BitPacker<String>) -> Result<(), &'static str> {
        let new_bits_len: usize = self.bits_len + bit_packer.bits_len;
        self.bytes_len = new_bits_len.div_ceil(8);

        while self.bits.len() < self.bytes_len {
            self.bits.insert(0, 0);
        }

        for i in 0..bit_packer.bits_len {
            let byte_idx: usize = self.bytes_len - 1 - (i + self.bits_len).div_euclid(8);
            let bit_idx: usize = (i + self.bits_len) % 8;

            if bit_packer[i as u8] == 1 {
                println!("ejkng");
                self.bits[byte_idx] |= 1 << bit_idx;
            }
        }

        self.bits_len = new_bits_len;
        
        self.value = BitPacker::parse_str(self.bits()).unwrap().value; // A refactor

       Ok(())
    }
}