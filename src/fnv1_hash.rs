// Copyright (c) Evan Overman 2023 (https://an-prata.it).
// Licensed under the MIT License.
// See LICENSE file in repository root for full text.

//! The FNV-1 hashing algorithm is a non-cryptographic hash function chosen here
//! for its ease of implementation. Based off the contents of this
//! [Wikipidia article](https://en.wikipedia.org/wiki/Fowler-Noll-Vo_hash_function)

const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
const FNV_PRIME: u64 = 1099511628211;

pub trait Hashable {
    /// Calculates the FNV-1 hash on `self`.
    fn hash(self) -> u64;
}

impl<T> Hashable for T
where
    T: IntoIterator<Item = u8>,
{
    fn hash(self) -> u64 {
        hash(self)
    }
}

/// Performs an FNV-1 hash on the given bytes and returns the result.
#[must_use]
pub fn hash<T>(bytes: T) -> u64
where
    T: IntoIterator<Item = u8>,
{
    bytes.into_iter().fold(FNV_OFFSET_BASIS, |acc, i| {
        lower_byte_xor(acc.wrapping_mul(FNV_PRIME), i)
    })
}

/// Performs a XOR operation between the lowest byte of the [`u64`] and the
/// given [`u8`], then returns a [`u64`] with its higher bytes unmodified.
///
/// [`u64`]: u64
/// [`u8`]: u8
#[inline]
#[must_use]
fn lower_byte_xor(a: u64, b: u8) -> u64 {
    let lowest = (a & u8::MAX as u64) as u8;
    a & !(u8::MAX as u64) | (lowest ^ b) as u64
}

#[cfg(test)]
mod tests {
    use super::Hashable;

    #[test]
    fn check_hash_differences() {
        let a: [u8; 6] = [32, 45, 234, 58, 72, 37];
        let b: [u8; 6] = [23, 43, 127, 32, 32, 123];

        assert_ne!(a.hash(), b.hash());
        assert_eq!(a.hash(), a.clone().hash());
    }
}
