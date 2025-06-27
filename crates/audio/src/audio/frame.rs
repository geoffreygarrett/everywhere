//! Strongly‑typed fixed‑size PCM frame (20 ms @ 48 kHz mono)

use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

#[repr(transparent)]
pub struct Frame<const N: usize>([i16; N]);

impl<const N: usize> Default for Frame<N> {
    fn default() -> Self { Self(unsafe { MaybeUninit::zeroed().assume_init() }) }
}
impl<const N: usize> Deref for Frame<N> {
    type Target = [i16; N];
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<const N: usize> DerefMut for Frame<N> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
