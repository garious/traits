//! This crate provides trait for Message Authentication Code (MAC) algorithms.
#![no_std]
extern crate constant_time_eq;
extern crate generic_array;

use constant_time_eq::constant_time_eq;
use generic_array::{GenericArray, ArrayLength};
use generic_array::typenum::Unsigned;

#[cfg(feature = "dev")]
pub mod dev;

/// Error type for signaling failed MAC verification
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct MacError;

/// Error type for signaling invalid key length for MAC initialization
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct InvalidKeyLength;

/// The `Mac` trait defines methods for a Message Authentication algorithm.
pub trait Mac: core::marker::Sized {
    type OutputSize: ArrayLength<u8>;
    type KeySize: ArrayLength<u8>;

    /// Create a new MAC instance from key with fixed size.
    fn new(key: &GenericArray<u8, Self::KeySize>) -> Self;

    /// Create a new MAC instance from variable sized key.
    fn new_varkey(key: &[u8]) -> Result<Self, InvalidKeyLength> {
        if key.len() != Self::KeySize::to_usize() {
            Err(InvalidKeyLength)
        } else {
            Ok(Self::new(GenericArray::from_slice(key)))
        }
    }

    /// Process input data.
    fn input(&mut self, data: &[u8]);

    /// Obtain the result of a `Mac` computation as a `MacResult` and reset
    /// `Mac` instance.
    fn result(&mut self) -> MacResult<Self::OutputSize>;

    /// Check if code is correct for the processed input and reset
    /// `Mac` instance.
    fn verify(&mut self, code: &[u8]) -> Result<(), MacError> {
        if Self::OutputSize::to_usize() != code.len() {
            Err(MacError)
        } else {
            let result = MacResult::new(GenericArray::clone_from_slice(code));
            if result != self.result() {
                Err(MacError)
            } else {
                Ok(())
            }
        }
    }
}

/// `MacResult` is a thin wrapper around bytes array which provides a safe `Eq`
/// implementation that runs in a fixed time.
#[derive(Clone)]
pub struct MacResult<N: ArrayLength<u8>> {
    code: GenericArray<u8, N>
}

impl<N> MacResult<N> where N: ArrayLength<u8> {
    /// Create a new MacResult.
    pub fn new(code: GenericArray<u8, N>) -> MacResult<N> {
        MacResult { code }
    }

    /// Get the code value as a bytes array. Be very careful using this method,
    /// since incorrect use of the code value may permit timing attacks which
    /// defeat the security provided by the `Mac` trait.
    pub fn code(self) -> GenericArray<u8, N> {
        self.code
    }
}

impl<N> PartialEq for MacResult<N> where N: ArrayLength<u8> {
    fn eq(&self, x: &MacResult<N>) -> bool {
        constant_time_eq(&self.code[..], &x.code[..])
    }
}

impl<N> Eq for MacResult<N> where N: ArrayLength<u8> { }
