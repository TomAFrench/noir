use crate::ops::{Add, Sub, Mul, Div, Rem,};


global bn254_fq = [0x47, 0xFD, 0x7C, 0xD8, 0x16, 0x8C, 0x20, 0x3C, 0x8d, 0xca, 0x71, 0x68, 0x91, 0x6a, 0x81, 0x97,
                   0x5d, 0x58, 0x81, 0x81, 0xb6, 0x45, 0x50, 0xb8, 0x29, 0xa0, 0x31, 0xe1, 0x72, 0x4e, 0x64, 0x30];
global bn254_fr = [0x01, 0x00, 0x00, 0x00, 0x3F, 0x59, 0x1F, 0x43, 0x09, 0x97, 0xB9, 0x79, 0x48, 0xE8, 0x33, 0x28, 
                   0x5D, 0x58, 0x81, 0x81, 0xB6, 0x45, 0x50, 0xB8, 0x29, 0xA0, 0x31, 0xE1, 0x72, 0x4E, 0x64, 0x30];
global secpk1_fr = [0x41, 0x41, 0x36, 0xD0, 0x8C, 0x5E, 0xD2, 0xBF, 0x3B, 0xA0, 0x48, 0xAF, 0xE6, 0xDC, 0xAE, 0xBA,
                    0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
global secpk1_fq = [0x2F, 0xFC, 0xFF, 0xFF, 0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
global secpr1_fq = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF];
global secpr1_fr = [0x51, 0x25, 0x63, 0xFC, 0xC2, 0xCA, 0xB9, 0xF3, 0x84, 0x9E, 0x17, 0xA7, 0xAD, 0xFA, 0xE6, 0xBC,
                    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,0xFF, 0xFF, 0xFF, 0xFF];
 

struct BigInt {
    pointer: u32,
    modulus: u32,
}

impl BigInt {
    #[builtin(bigint_add)]
    fn bigint_add(self, other: BigInt) -> BigInt {
    }
    #[builtin(bigint_sub)]
    fn bigint_sub(self, other: BigInt) -> BigInt {
    }
    #[builtin(bigint_mul)]
    fn bigint_mul(self, other: BigInt) -> BigInt {
    }
    #[builtin(bigint_div)]
    fn bigint_div(self, other: BigInt) -> BigInt {
    }
    #[builtin(bigint_from_le_bytes)]
    fn from_le_bytes(bytes: [u8], modulus: [u8]) -> BigInt {}
    #[builtin(bigint_to_le_bytes)]
    pub fn to_le_bytes(self) -> [u8] {}

    pub fn bn254_fr_from_le_bytes(bytes: [u8]) -> BigInt {
        BigInt::from_le_bytes(bytes, bn254_fr)
    }
    pub fn bn254_fq_from_le_bytes(bytes: [u8]) -> BigInt {
        BigInt::from_le_bytes(bytes, bn254_fq)
    }
    pub fn secpk1_fq_from_le_bytes(bytes: [u8]) -> BigInt {
        BigInt::from_le_bytes(bytes, secpk1_fq)
    }
    pub fn secpk1_fr_from_le_bytes(bytes: [u8]) -> BigInt {
        BigInt::from_le_bytes(bytes, secpk1_fr)
    }
}

impl Add for BigInt { 
    fn add(self: Self, other: BigInt) -> BigInt {
        self.bigint_add(other)
    }
}
impl Sub for BigInt { 
    fn sub(self: Self, other: BigInt) -> BigInt {
        self.bigint_sub(other)
    }
}
impl Mul for BigInt { 
    fn mul(self: Self, other: BigInt) -> BigInt {
        self.bigint_mul(other)
    }
}
impl Div for BigInt { 
    fn div(self: Self, other: BigInt) -> BigInt {
        self.bigint_div(other)
    }
}
impl Rem for BigInt { 
    fn rem(self: Self, other: BigInt) -> BigInt {
        let quotient = self.bigint_div(other);
        self.bigint_sub(quotient.bigint_mul(other))
    }
}

