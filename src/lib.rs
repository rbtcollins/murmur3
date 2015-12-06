extern crate byteorder;


use std::io::Cursor;
use std::io::Read;
use byteorder::{LittleEndian, ReadBytesExt, ByteOrder};


// TODO: test with using chunks
// https://doc.rust-lang.org/beta/std/slice/struct.Chunks.html

pub fn murmur3_32(source: &mut Read, seed: u32) -> u32 {
    const C1: u32 = 0x85ebca6b;
    const C2: u32 = 0xc2b2ae35;
    const R1: u32 = 16;
    const R2: u32 = 13;
    const M: u32 = 5;
    const N: u32 = 0xe6546b64;
    let mut hash = seed;
    let mut buf = [0; 4];
    let mut processed: u32 = 0;
    loop {
        match source.read(&mut buf[..]) {
            Ok(size) => {
                match size {
                    4 => {
                        let mut tmp = Cursor::new(buf);
                        let k = tmp.read_u32::<LittleEndian>().unwrap();
                        hash ^= calc_k(k);
                        hash = hash.rotate_left(R2);
                        hash = (hash.wrapping_mul(M)).wrapping_add(N);
                    }
                    3 => {
                        let k: u32 = ((buf[2] as u32) << 16) | ((buf[1] as u32) << 8) |
                                     (buf[0] as u32);
                        hash ^= calc_k(k);
                    }
                    2 => {
                        let k: u32 = ((buf[1] as u32) << 8) | (buf[0] as u32);
                        hash ^= calc_k(k);
                    }
                    1 => {
                        let k: u32 = buf[0] as u32;
                        hash ^= calc_k(k);
                    }
                    0 => {
                        hash ^= (processed) as u32;
                        hash ^= hash.wrapping_shr(R1);
                        hash = hash.wrapping_mul(C1);
                        hash ^= hash.wrapping_shr(R2);
                        hash = hash.wrapping_mul(C2);
                        hash ^= hash.wrapping_shr(R1);
                        return hash;
                    }
                    _ => panic!("Invalid read size!"),
                };
                processed += size as u32;
            }
            Err(e) => panic!(e),
        }
    }
}

fn calc_k(k: u32) -> u32 {
    const C1: u32 = 0xcc9e2d51;
    const C2: u32 = 0x1b873593;
    const R1: u32 = 15;
    k.wrapping_mul(C1).rotate_left(R1).wrapping_mul(C2)
}


pub fn murmur3_x64_128(source: &mut Read, seed: u32, out: &mut [u8]) {
    const C1: u64 = 0x52dce729;
    const C2: u64 = 0x38495ab5;
    const R1: u32 = 27;
    const R2: u32 = 31;
    const M: u64 = 5;
    let mut h1: u64 = seed as u64;
    let mut h2: u64 = seed as u64;
    let mut buf = [0; 16];
    let mut processed: u32 = 0;
    if out.len() < 16 {
        panic!("Invalid out buffer size");
    }
    loop {
        match source.read(&mut buf[..]) {
            Ok(size) => {
                let mut cur = Cursor::new(buf);
                match size {
                    16 => {
                        let k1 = cur.read_u64::<LittleEndian>().unwrap();
                        let k2 = cur.read_u64::<LittleEndian>().unwrap();
                        h1 ^= process_h1_k(k1);
                        h1 = h1.rotate_left(R1).wrapping_add(h2).wrapping_mul(M).wrapping_add(C1);
                        h2 ^= process_h2_k(k2);
                        h2 = h2.rotate_left(R2).wrapping_add(h1).wrapping_mul(M).wrapping_add(C2);
                    }
                    9...15 => {
                        h1 ^= process_h1_k(cur.read_u64::<LittleEndian>().unwrap());
                        h2 ^= process_h2_k(LittleEndian::read_int(&buf[8..], size - 8) as u64);
                    }
                    8 => {
                        h1 ^= process_h1_k(cur.read_u64::<LittleEndian>().unwrap());
                    }
                    2...7 => {
                        h1 ^= process_h1_k(LittleEndian::read_int(&buf, size) as u64);
                    }
                    1 => {
                        h1 ^= process_h1_k(buf[0] as u64);
                    }
                    0 => {
                        h1 ^= processed as u64;
                        h2 ^= processed as u64;
                        h1 = h1.wrapping_add(h2);
                        h2 = h2.wrapping_add(h1);
                        h1 = fmix64(h1);
                        h2 = fmix64(h2);
                        h1 = h1.wrapping_add(h2);
                        h2 = h2.wrapping_add(h1);
                        LittleEndian::write_u64(&mut out[0..], h1);
                        LittleEndian::write_u64(&mut out[8..], h2);
                        return;
                    }
                    _ => {
                        panic!("Invalid read");
                    }
                }
                processed += size as u32;
            }
            Err(e) => panic!(e),
        }
    }
}

fn process_h1_k(k: u64) -> u64 {
    const C1: u64 = 0x87c37b91114253d5;
    const C2: u64 = 0x4cf5ad432745937f;
    const R: u32 = 31;
    k.wrapping_mul(C1).rotate_left(R).wrapping_mul(C2)
}

fn process_h2_k(k: u64) -> u64 {
    const C1: u64 = 0x87c37b91114253d5;
    const C2: u64 = 0x4cf5ad432745937f;
    const R: u32 = 33;
    k.wrapping_mul(C2).rotate_left(R).wrapping_mul(C1)
}

fn fmix64(k: u64) -> u64 {
    const C1: u64 = 0xff51afd7ed558ccd;
    const C2: u64 = 0xc4ceb9fe1a85ec53;
    const R: u32 = 33;
    let mut tmp = k;
    tmp ^= tmp >> R;
    tmp = tmp.wrapping_mul(C1);
    tmp ^= tmp >> R;
    tmp = tmp.wrapping_mul(C2);
    tmp ^= tmp >> R;
    tmp
}