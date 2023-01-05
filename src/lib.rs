use std::{borrow::Cow, ptr};

use phf::phf_map;

use thiserror::Error;

const XOR_CODE: u64 = 23442827791579;
const MASK_CODE: u64 = 2251799813685247;

pub const MAX_AID: u64 = 1 << 51;
pub const MIN_AID: u64 = 1;

const BASE: u64 = 58;
const BV_LEN: usize = 12;
const PREFIX: &str = "BV1";

const ALPHABET: [u8; BASE as usize] = [
  b'F', b'c', b'w', b'A', b'P', b'N', b'K', b'T', b'M', b'u', b'g', b'3', b'G', b'V', b'5', b'L',
  b'j', b'7', b'E', b'J', b'n', b'H', b'p', b'W', b's', b'x', b'4', b't', b'b', b'8', b'h', b'a',
  b'Y', b'e', b'v', b'i', b'q', b'B', b'z', b'6', b'r', b'k', b'C', b'y', b'1', b'2', b'm', b'U',
  b'S', b'D', b'Q', b'X', b'9', b'R', b'd', b'o', b'Z', b'f',
];

const REVERSE: phf::Map<u8, usize> = phf_map! {
  b'F' => 0,  b'c' => 1,  b'w' => 2,  b'A' => 3,  b'P' => 4,  b'N' => 5,  b'K' => 6,  b'T' => 7,  b'M' => 8,
  b'u' => 9,  b'g' => 10, b'3' => 11, b'G' => 12, b'V' => 13, b'5' => 14, b'L' => 15, b'j' => 16, b'7' => 17,
  b'E' => 18, b'J' => 19, b'n' => 20, b'H' => 21, b'p' => 22, b'W' => 23, b's' => 24, b'x' => 25, b'4' => 26,
  b't' => 27, b'b' => 28, b'8' => 29, b'h' => 30, b'a' => 31, b'Y' => 32, b'e' => 33, b'v' => 34, b'i' => 35,
  b'q' => 36, b'B' => 37, b'z' => 38, b'6' => 39, b'r' => 40, b'k' => 41, b'C' => 42, b'y' => 43, b'1' => 44,
  b'2' => 45, b'm' => 46, b'U' => 47, b'S' => 48, b'D' => 49, b'Q' => 50, b'X' => 51, b'9' => 52, b'R' => 53,
  b'd' => 54, b'o' => 55, b'Z' => 56, b'f' => 57,
};

#[derive(Error, Debug, PartialEq)]
pub enum Error {
  #[error("Av {0} is smaller than {MIN_AID}")]
  AvTooSmall(u64),
  #[error("Av {0} is bigger than {MAX_AID}")]
  AvTooBig(u64),
  #[error("Bv is empty")]
  BvEmpty,
  #[error("Bv is too small")]
  BvTooSmall,
  #[error("Bv is too big")]
  BvTooBig,
  #[error("Bv prefix should be ignore-cased `BV1`")]
  BvInvalidPrefix,
  #[error("Bv is invalid, with invalid char code `{0}`")]
  BvInvalidChar(char),
  #[error("Bv with unicode char")]
  BvWithUnicode,
}

pub fn av2bv(avid: u64) -> Result<String, Error> {
  if avid < MIN_AID {
    return Err(Error::AvTooSmall(avid));
  }
  if avid >= MAX_AID {
    return Err(Error::AvTooBig(avid));
  }

  let mut bytes: [u8; BV_LEN] = [
    b'B', b'V', b'1', b'0', b'0', b'0', b'0', b'0', b'0', b'0', b'0', b'0',
  ];

  let mut bv_idx = BV_LEN - 1;
  let mut tmp = (MAX_AID | avid) ^ XOR_CODE;
  while tmp != 0 {
    let table_idx = tmp % BASE;
    // SAFETY: a positive number mod 58 is in 0..58
    let part = unsafe { ALPHABET.get_unchecked(table_idx as usize) };
    unsafe {
      let ele = bytes.get_unchecked_mut(bv_idx);
      *ele = *part;
    }
    tmp = tmp / BASE;
    bv_idx -= 1;
  }

  // SAFETY, 3 < 4 < 7 < 9 < BV_LEN
  unsafe {
    unchecked_swap(&mut bytes, 3, 9);
    unchecked_swap(&mut bytes, 4, 7);
  }

  // SAFETY: bytes represent an ASCII string
  let str = unsafe { String::from_utf8_unchecked(bytes.to_vec()) };

  Ok(str)
}

pub fn bv2av<'a, S>(bvid: S) -> Result<u64, Error>
where
  S: Into<Cow<'a, str>>,
{
  let bvid: Cow<_> = bvid.into();
  if bvid.is_empty() {
    return Err(Error::BvEmpty);
  }

  if !bvid.is_ascii() {
    return Err(Error::BvWithUnicode);
  }

  match bvid.as_bytes().len().cmp(&BV_LEN) {
    std::cmp::Ordering::Less => return Err(Error::BvTooSmall),
    std::cmp::Ordering::Greater => return Err(Error::BvTooBig),
    _ => {},
  }

  // SAFETY: Already checked before
  let prefix = unsafe { bvid.get_unchecked(0..3) };
  if !prefix.eq_ignore_ascii_case(PREFIX) {
    return Err(Error::BvInvalidPrefix);
  }

  let mut bvid = match bvid {
    Cow::Borrowed(str) => str.to_string(),
    Cow::Owned(string) => string,
  };

  unsafe {
    let bv_vec = bvid.as_mut_vec();
    unchecked_swap(bv_vec, 3, 9);
    unchecked_swap(bv_vec, 4, 7);
  }

  let mut tmp = 0;

  for byte in &bvid.as_bytes()[3..] {
    let Some(idx) = REVERSE.get(byte) else {
      return Err(Error::BvInvalidChar(*byte as char));
    };
    tmp = tmp * BASE + *idx as u64;
  }
 
  // Equivalence of: format!("{:b}", tmp).size()
  let bin_len = if tmp == 0 {
    0
  } else {
    u64::BITS - tmp.leading_zeros()
  };

  if bin_len > 52 {
    return Err(Error::BvTooBig);
  }

  if bin_len < 52 {
    return Err(Error::BvTooSmall);
  }

  let avid = (tmp & MASK_CODE) ^ XOR_CODE;

  if avid < MIN_AID {
    return Err(Error::BvTooSmall);
  }

  Ok(avid)
}

unsafe fn unchecked_swap<I>(array: &mut [I], index_a: usize, index_b: usize) {
  let pa = ptr::addr_of_mut!(array[index_a]);
  let pb = ptr::addr_of_mut!(array[index_b]);
  unsafe {
    ptr::swap(pa, pb);
  }
}
