# abv

AV and BV convert functions for Bilibili videos.

## Advantage

Unlike other libraries, numbers up to `2251799813685248` are supported.

## Usage

```rust
bv2av("BV1B8Ziyo7s2").unwrap(); // 1145141919810
av2bv(11451419180).unwrap();    // "BV1gA4v1m7BV"
```

## License

Licensed under either of

- MIT License ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)
- CC0 License/Waiver, ([LICENSE-CC0](LICENSE-CC0) or <https://creativecommons.org/publicdomain/zero/1.0/legalcode>)

at your option.
