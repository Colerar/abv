use std::{borrow::Cow, ptr};

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

#[rustfmt::skip]
fn rev(value: u8) -> Option<u8> {
  use std::option::Option::Some as S;
  match value {
    b'F' => S(0),  b'c' => S(1),  b'w' => S(2),  b'A' => S(3),  b'P' => S(4),  b'N' => S(5),  b'K' => S(6),  b'T' => S(7),  b'M' => S(8),
    b'u' => S(9),  b'g' => S(10), b'3' => S(11), b'G' => S(12), b'V' => S(13), b'5' => S(14), b'L' => S(15), b'j' => S(16), b'7' => S(17),
    b'E' => S(18), b'J' => S(19), b'n' => S(20), b'H' => S(21), b'p' => S(22), b'W' => S(23), b's' => S(24), b'x' => S(25), b'4' => S(26),
    b't' => S(27), b'b' => S(28), b'8' => S(29), b'h' => S(30), b'a' => S(31), b'Y' => S(32), b'e' => S(33), b'v' => S(34), b'i' => S(35),
    b'q' => S(36), b'B' => S(37), b'z' => S(38), b'6' => S(39), b'r' => S(40), b'k' => S(41), b'C' => S(42), b'y' => S(43), b'1' => S(44),
    b'2' => S(45), b'm' => S(46), b'U' => S(47), b'S' => S(48), b'D' => S(49), b'Q' => S(50), b'X' => S(51), b'9' => S(52), b'R' => S(53),
    b'd' => S(54), b'o' => S(55), b'Z' => S(56), b'f' => S(57),
    _ => None
  }
}

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
    tmp /= BASE;
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
    let Some(idx) = rev(*byte) else {
      return Err(Error::BvInvalidChar(*byte as char));
    };
    tmp = tmp * BASE + idx as u64;
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
