use ciphers::gost_28147_89::{
    feedback_gamma_decode, feedback_gamma_encode, simple_repl_decode, simple_repl_encode,
    straight_gamma_decode, straight_gamma_encode, Key,
};

fn main() {
    let mut data = b"Hello world, this is a message that is hopefully sufficently long to make encoding & decoding work".to_vec();
    let key = Key([
        0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
    ]);
    let sync = [0xa015, 0xb127];
    for byte in &data {
        print!("{byte:02x}");
    }
    println!();
    feedback_gamma_encode(sync, &mut data, key);
    for byte in &data {
        print!("{byte:02x}");
    }
    println!();
    feedback_gamma_decode(sync, &mut data, key);
    for byte in &data {
        print!("{byte:02x}");
    }
    println!("\n{}", String::from_utf8(data).unwrap());
}
