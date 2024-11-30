#[derive(Clone, Copy)]
pub struct Key(pub [u32; 8]);

const C1: u32 = 0b0000_0001_0000_0001_0000_0001_0000_0001;
const C2: u32 = 0b0000_0001_0000_0001_0000_0001_0000_0100;

pub struct State {
    x_key: [u32; 8],
    n_acc: [u32; 4],
    c_sum: [u32; 4],
    k_blk: [[u64; 2]; 4],
}

impl Default for State {
    fn default() -> Self {
        Self {
            x_key: [0; 8],
            n_acc: [0; 4],
            c_sum: [0; 4],
            k_blk: [
                [0x0123456789ABCDEF, 0x1123456789ABCDEF],
                [0x2123456789ABCDEF, 0x3123456789ABCDEF],
                [0x4123456789ABCDEF, 0x5123456789ABCDEF],
                [0x6123456789ABCDEF, 0x7123456789ABCDEF],
            ],
        }
    }
}

impl State {
    fn with(key: Key) -> Self {
        Self {
            x_key: key.0,
            ..Default::default()
        }
    }
}

impl State {
    pub fn subs_2(&self, inpt: u32) -> u32 {
        u32::from_le_bytes(
            inpt.to_le_bytes()
                .iter()
                .enumerate()
                .map(|(i, b)| {
                    let first = b & 0x0F;
                    let secnd = (b & 0xF0) >> 4;
                    let first = ((self.k_blk[i][0] >> (60 - first * 4)) & 0x0F) as u8;
                    let secnd = ((self.k_blk[i][1] >> (60 - secnd * 4)) & 0x0F) as u8;
                    (secnd << 4) | first
                })
                .collect::<Vec<u8>>()
                .try_into() // can do since we know, that result is 4 bytes long
                .unwrap(),
        )
    }

    pub fn substitute(&self, inpt: u32) -> u32 {
        let mut bytes = inpt.to_le_bytes();

        bytes.iter_mut().enumerate().for_each(|(i, b)| {
            let first = *b & 0x0F;
            let secnd = *b & 0xF0;
            let first = ((self.k_blk[i][0] << (first * 4) >> 60) & 0x0F) as u8;
            let secnd = ((self.k_blk[i][1] << (secnd / 4) >> 60) & 0x0F) as u8;
            *b = (secnd << 4) | first
        });

        u32::from_le_bytes(bytes)
    }
}
