use ciphers::gost_28147_89::{simple_repl_decode, simple_repl_encode, Key};

fn main() {
    let mut data = b"Hello world, this is a message that is hopefully sufficently long to make encoding & decoding work".to_vec();
    let key = Key([
        0x0123, 0x4567, 0x89AB, 0xCDEF, 0x0123, 0x4567, 0x89AB, 0xCDEF,
    ]);
    for byte in &data {
        print!("{byte:02x}");
    }
    println!();
    simple_repl_encode(&mut data, key);
    for byte in &data {
        print!("{byte:02x}");
    }
    println!();
    simple_repl_decode(&mut data, key);
    for byte in &data {
        print!("{byte:02x}");
    }
    println!("\n{}", String::from_utf8(data).unwrap());
}
