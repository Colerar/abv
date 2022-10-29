use phf::phf_map;

use thiserror::Error;

const XOR_CODE: u64 = 23442827791579;
const MASK_CODE: u64 = 2251799813685247;

pub const MAX_AID: u64 = 1 << 51;
pub const MIN_AID: u64 = 1;

const BASE: u64 = 58;
const PREFIX: &str = "BV1";

const ALPHABET: [char; BASE as usize] = [
  'F', 'c', 'w', 'A', 'P', 'N', 'K', 'T', 'M', 'u', 'g', '3', 'G', 'V', '5', 'L', 'j', '7', 'E',
  'J', 'n', 'H', 'p', 'W', 's', 'x', '4', 't', 'b', '8', 'h', 'a', 'Y', 'e', 'v', 'i', 'q', 'B',
  'z', '6', 'r', 'k', 'C', 'y', '1', '2', 'm', 'U', 'S', 'D', 'Q', 'X', '9', 'R', 'd', 'o', 'Z',
  'f',
];

const REVERSE: phf::Map<char, usize> = phf_map! {
  'F' => 0, 'c' => 1, 'w' => 2, 'A' => 3, 'P' => 4, 'N' => 5, 'K' => 6, 'T' => 7, 'M' => 8, 'u' => 9,
  'g' => 10, '3' => 11, 'G' => 12, 'V' => 13, '5' => 14, 'L' => 15, 'j' => 16, '7' => 17, 'E' => 18,
  'J' => 19, 'n' => 20, 'H' => 21, 'p' => 22, 'W' => 23, 's' => 24, 'x' => 25, '4' => 26, 't' => 27,
  'b' => 28, '8' => 29, 'h' => 30, 'a' => 31, 'Y' => 32, 'e' => 33, 'v' => 34, 'i' => 35, 'q' => 36,
  'B' => 37, 'z' => 38, '6' => 39, 'r' => 40, 'k' => 41, 'C' => 42, 'y' => 43, '1' => 44, '2' => 45,
  'm' => 46, 'U' => 47, 'S' => 48, 'D' => 49, 'Q' => 50, 'X' => 51, '9' => 52, 'R' => 53, 'd' => 54,
  'o' => 55, 'Z' => 56, 'f' => 57,
};

#[derive(Error, Debug)]
pub enum Error {
  #[error("Av {0} is smaller than {MIN_AID}")]
  AvTooSmall(u64),
  #[error("Av {0} is bigger than {MAX_AID}")]
  AvTooBig(u64),
  #[error("Bv is empty")]
  BvEmpty,
  #[error("Bv is too small: {0}")]
  BvTooSmall(String),
  #[error("Bv is too big: {0}")]
  BvTooBig(String),
  #[error("Bv `{0}` is illegal, with invalid char code `{1}`")]
  BvInvalidChar(String, u8),
  #[error("Bv with unicode char")]
  BvWithUnicode,
}

pub fn av2bv(avid: u64) -> Result<String, Error> {
  if avid < MIN_AID {
    Err(Error::AvTooSmall(avid))?
  }
  if avid >= MAX_AID {
    Err(Error::AvTooBig(avid))?
  }

  let mut bv = String::new();
  let mut tmp = (MAX_AID | avid) ^ XOR_CODE;
  while tmp != 0 {
    // SAFETY: a positive number mod 58 is in 0..58
    let part = unsafe {
      let idx = tmp % BASE;
      ALPHABET.get_unchecked(idx as usize)
    };
    bv = format!("{}{}", part, bv);
    tmp = tmp / BASE;
  }

  // SAFETY: bv is a ASCII string
  let bv_vec = unsafe { bv.as_mut_vec() };

  let tmp = bv_vec[0];
  bv_vec[0] = bv_vec[6];
  bv_vec[6] = tmp;
  let tmp = bv_vec[1];
  bv_vec[1] = bv_vec[4];
  bv_vec[4] = tmp;

  Ok(format!("{}{}", PREFIX, bv))
}

pub fn bv2av(bvid: &str) -> Result<u64, Error> {
  if bvid.is_empty() {
    Err(Error::BvEmpty)?
  }
  let r_bvid = bvid;
  let mut bvid = bvid;

  if bvid.len() != bvid.chars().count() {
    Err(Error::BvWithUnicode)?
  }

  if bvid.len() == 10 && bvid.starts_with("1") {
    bvid = &bvid[1..];
  } else if bvid.len() > 3 && bvid.starts_with(&bvid[0..3].to_ascii_uppercase()) {
    bvid = &bvid[3..];
  }

  if bvid.len() < 9 {
    Err(Error::BvTooSmall(r_bvid.to_string()))?
  }

  let mut bvid = bvid.to_string();

  {
    let bv_vec = unsafe { bvid.as_mut_vec() };

    let tmp = bv_vec[1];
    bv_vec[1] = bv_vec[4];
    bv_vec[4] = tmp;
    let tmp = bv_vec[0];
    bv_vec[0] = bv_vec[6];
    bv_vec[6] = tmp;
  }

  let mut tmp = 0;

  for byte in bvid.bytes() {
    let char = byte as char;
    let index = if let Some(idx) = REVERSE.get(&char) {
      idx
    } else {
      Err(Error::BvInvalidChar(r_bvid.to_string(), byte))?
    };
    tmp = tmp * BASE + *index as u64;
  }

  // Equivalence of: format!("{:b}", tmp).size()
  let bin_len = if tmp == 0 {
    0
  } else {
    u64::BITS - tmp.leading_zeros()
  };

  if bin_len > 52 {
    Err(Error::BvTooBig(r_bvid.to_string()))?;
  }

  if bin_len < 52 {
    Err(Error::BvTooSmall(r_bvid.to_string()))?;
  }

  let avid = (tmp & MASK_CODE) ^ XOR_CODE;

  if avid < MIN_AID {
    Err(Error::BvTooSmall(r_bvid.to_string()))?;
  }

  Ok(avid)
}

#[cfg(test)]
mod tests {
  use crate::{av2bv, bv2av, Error};

  #[test]
  fn av2bv_test() {
    assert_eq!("BV1gA4v1m7BV", av2bv(11451419180).unwrap());

    match av2bv(0).unwrap_err() {
      Error::AvTooSmall(_) => { /* pass */ },
      _ => panic!(),
    }

    match av2bv(99999999999999999).unwrap_err() {
      Error::AvTooBig(_) => { /* pass */ },
      _ => panic!(),
    }
  }

  #[test]
  fn bv2av_test() {
    assert_eq!(1145141919810, bv2av("BV1B8Ziyo7s2").unwrap());

    match bv2av("BV测试").unwrap_err() {
      Error::BvWithUnicode => { /* pass */ },
      _ => panic!(),
    }
  }
}
