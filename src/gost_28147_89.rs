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
    let mut r = (Wrapping(a) + Wrapping(b) + Wrapping(1)).0;
    if u32::MAX - b > a {
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

fn round_encode(state: &mut State) {
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

    state.n_acc.swap(0, 1);
}

fn round_decode(state: &mut State) {
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

    state.n_acc.swap(0, 1);
}

pub fn simple_repl_encode(data: &mut [u8], key: Key) {
    let mut state = State::with(key);

    for block in data.chunks_exact_mut(8) {
        // can do unwrap since we are in chunks of 8
        let block: &mut [u8; 8] = block.try_into().unwrap();
        [state.n_acc[0], state.n_acc[1]] = u32x2_from_be(*block);
        round_encode(&mut state);
        *block = u32x2_to_be([state.n_acc[0], state.n_acc[1]]);
    }
}

pub fn simple_repl_decode(data: &mut [u8], key: Key) {
    let mut state = State::with(key);

    for block in data.chunks_exact_mut(8) {
        // can do unwrap since we are in chunks of 8
        let block: &mut [u8; 8] = block.try_into().unwrap();
        [state.n_acc[0], state.n_acc[1]] = u32x2_from_be(*block);
        round_decode(&mut state);
        *block = u32x2_to_be([state.n_acc[0], state.n_acc[1]]);
    }
}

fn u32x2_from_be(x: [u8; 8]) -> [u32; 2] {
    let [a, b, c, d, e, f, g, h] = x;
    [[a, b, c, d], [e, f, g, h]].map(u32::from_be_bytes)
}

fn u32x2_to_be(x: [u32; 2]) -> [u8; 8] {
    let [[a, b, c, d], [e, f, g, h]] = x.map(u32::to_be_bytes);
    [a, b, c, d, e, f, g, h]
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
    let r = sum_m1(0xFFFFFFFF, 0x03);
    assert_eq!(r, 0x03);
    let r = sum_m1(0xFFFFFFFF, 0x00);
    assert_eq!(r, 0x00);
}

fn sum_t1(a: u32, b: u32) -> u32 {
    let r = a as u64 + b as u64;
    (r % u32::MAX as u64) as u32
}

//this one emits less ams instructions
fn sum_t2(a: u32, b: u32) -> u32 {
    let mut r = (Wrapping(a) + Wrapping(b) + Wrapping(1)).0;
    if u32::MAX - b > a {
        r -= 1;
    }
    r
}

#[test]
fn cmp_summ() {
    for a in u32::MAX - 20..=u32::MAX {
        for b in 0..16 {
            assert_eq!(sum_t1(a, b), sum_t2(a, b));
        }
    }
}
