pub struct FixedStr<const BYTES: usize> {
    bytes: [u8; BYTES]
}