#![feature(test)]

use test::Bencher;

extern crate test;

use abv::*;

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

  assert_eq!(bv2av(""), Err(Error::BvEmpty));
  assert_eq!(bv2av("   "), Err(Error::BvTooSmall));
  assert_eq!(bv2av("BV测试"), Err(Error::BvWithUnicode));
  assert_eq!(bv2av("BV2B8Ziyo7s2"), Err(Error::BvInvalidPrefix));
  assert_eq!(bv2av("BV212312"), Err(Error::BvTooSmall));
  assert_eq!(bv2av("BV2122222222312"), Err(Error::BvTooBig));
  assert_eq!(bv2av("BV1B0Ziyo7s2"), Err(Error::BvInvalidChar('0')));
}

#[bench]
fn av2bv_bench(b: &mut Bencher) {
  b.iter(|| av2bv(1700001))
}

#[bench]
fn bv2av_bench(b: &mut Bencher) {
  b.iter(|| bv2av("BV17x411w7KC"))
}
