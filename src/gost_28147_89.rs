use std::num::Wrapping;

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

fn sum_m0(a: u32, b: u32) -> u32 {
    (Wrapping(a) + Wrapping(b)).0
}

fn sum_m1(a: u32, b: u32) -> u32 {
    let mut r = (Wrapping(a) + Wrapping(b)).0 + 1;
    if u32::MAX - b >= a {
        r -= 1;
    }
    r
}

fn xor_32(a: u32, b: u32) -> u32 {
    a ^ b
}

fn xor_64(a: u64, b: u64) -> u64 {
    a ^ b
}

fn round_encode(block: &mut [u8; 8], state: &mut State) {
    let first = [block[0], block[1], block[2], block[3]];
    let secnd = [block[4], block[5], block[6], block[7]];
    state.n_acc[0] = u32::from_be_bytes(first);
    state.n_acc[1] = u32::from_be_bytes(secnd);

    for i in 0..24 {
        state.c_sum[0] = sum_m0(state.x_key[i % 8], state.n_acc[0]);
        let r = state.substitute(state.c_sum[0]).rotate_left(11);
        state.c_sum[1] = xor_32(state.n_acc[1], r);
        state.n_acc[1] = state.n_acc[0];
        state.n_acc[0] = state.c_sum[1];
    }
    for i in 24..32 {
        state.c_sum[0] = sum_m0(state.x_key[7 - (i % 8)], state.n_acc[0]);
        let r = state.substitute(state.c_sum[0]).rotate_left(11);
        state.c_sum[1] = xor_32(state.n_acc[1], r);
        state.n_acc[1] = state.n_acc[0];
        state.n_acc[0] = state.c_sum[1];
    }

    *block = (((state.n_acc[1] as u64) << 32) | state.n_acc[0] as u64).to_be_bytes()
}

fn round_decode(block: &mut [u8; 8], state: &mut State) {
    let first = [block[0], block[1], block[2], block[3]];
    let secnd = [block[4], block[5], block[6], block[7]];
    state.n_acc[0] = u32::from_be_bytes(first);
    state.n_acc[1] = u32::from_be_bytes(secnd);

    for i in 0..8 {
        state.c_sum[0] = sum_m0(state.x_key[i % 8], state.n_acc[0]);
        let r = state.substitute(state.c_sum[0]).rotate_left(11);
        state.c_sum[1] = xor_32(state.n_acc[1], r);
        state.n_acc[1] = state.n_acc[0];
        state.n_acc[0] = state.c_sum[1];
    }
    for i in 8..32 {
        state.c_sum[0] = sum_m0(state.x_key[7 - (i % 8)], state.n_acc[0]);
        let r = state.substitute(state.c_sum[0]).rotate_left(11);
        state.c_sum[1] = xor_32(state.n_acc[1], r);
        state.n_acc[1] = state.n_acc[0];
        state.n_acc[0] = state.c_sum[1];
    }

    *block = (((state.n_acc[1] as u64) << 32) | state.n_acc[0] as u64).to_be_bytes()
}

pub fn simple_repl_encode(data: &mut [u8], key: Key) {
    let mut state = State::with(key);

    for block in data.chunks_exact_mut(8) {
        // can do unwrap since we are in chunks of 8
        round_encode(block.try_into().unwrap(), &mut state)
    }
}

pub fn simple_repl_decode(data: &mut [u8], key: Key) {
    let mut state = State::with(key);

    for block in data.chunks_exact_mut(8) {
        // can do unwrap since we are in chunks of 8
        round_decode(block.try_into().unwrap(), &mut state)
    }
}

#[test]
fn eqeqeqeq() {
    let key = Key([
        0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
    ]);

    let state = State::with(key);
    let inpt = 0x01829732;
    let r1 = state.substitute(inpt);
    let r2 = state.subs_2(inpt);

    assert_eq!(r1, r2)
}

#[test]
fn summ() {
    let r = sum_m1(0x01, 0x0E);
    assert_eq!(r, 0x0F);
    let r = sum_m1(0xFFFFFFFF, 0x00000003);
    assert_eq!(r, 0x03);
}
