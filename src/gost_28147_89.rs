#[derive(Clone, Copy)]
pub struct Key(pub [u32; 8]);

const C1: u32 = 0b0000_0001_0000_0001_0000_0001_0000_0001;
const C2: u32 = 0b0000_0001_0000_0001_0000_0001_0000_0100;
