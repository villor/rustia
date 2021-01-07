use std::num::Wrapping;
use byteorder::{ByteOrder, LittleEndian};

const DELTA: Wrapping<u32> = Wrapping(0x9E3779B9);
const ROUNDS: u32 = 32;

// Port to Bytes to get rid of byteorder dependency? (or just bit shift)

pub fn encrypt(data: &mut [u8], key: &[Wrapping<u32>]) {
    if data.len() % 8 != 0 {
        panic!("xtea data not multiple of 8");
    }

    for block in data.chunks_mut(8) {
        let enc = encrypt_block(LittleEndian::read_u64(block), key);
        LittleEndian::write_u64(block, enc);
    }
}

fn encrypt_block(block: u64, key: &[Wrapping<u32>]) -> u64 {
    let mut v0 = Wrapping(block as u32);
    let mut v1 = Wrapping((block >> 32) as u32);
    let mut sum = Wrapping(0u32);

    for _ in 0..ROUNDS {
        v0 += ((v1 << 4 ^ v1 >> 5) + v1) ^ (sum + key[(sum.0 & 3) as usize]);
        sum += DELTA;
        v1 += ((v0 << 4 ^ v0 >> 5) + v0) ^ (sum + key[((sum.0 >> 11) & 3) as usize]);
    }

    (v1.0 as u64) << 32 | v0.0 as u64
}

pub fn decrypt(data: &mut [u8], key: &[Wrapping<u32>]) {
    if data.len() % 8 != 0 {
        panic!("xtea data not multiple of 8");
    }

    for block in data.chunks_mut(8) {
        let enc = decrypt_block(LittleEndian::read_u64(block), key);
        LittleEndian::write_u64(block, enc);
    }
}

fn decrypt_block(block: u64, key: &[Wrapping<u32>]) -> u64 {
    let mut v0 = Wrapping(block as u32);
    let mut v1 = Wrapping((block >> 32) as u32);
    let mut sum = DELTA * Wrapping(ROUNDS);

    for _ in 0..ROUNDS {
        v1 -= ((v0 << 4 ^ v0 >> 5) + v0) ^ (sum + key[((sum.0 >> 11) & 3) as usize]);
        sum -= DELTA;
        v0 -= ((v1 << 4 ^ v1 >> 5) + v1) ^ (sum + key[(sum.0 & 3) as usize]);
    }

    (v1.0 as u64) << 32 | v0.0 as u64
}

// TODO: decryption, tests, and profiling(optimization)
