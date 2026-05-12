use crate::{CollectionError, CollectionResult};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::slice;

#[derive(Debug)]
pub struct FixedVec<T, const N: usize> {
    elements: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedVec<T, N> {
    pub const fn new() -> Self {
        Self {
            elements: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.len == N
    }

    #[inline]
    pub fn push(&mut self, value: T) -> CollectionResult<()> {
        if self.len == N {
            return Err(CollectionError::FixedCapacityExceeded { capacity: N });
        }
        self.elements[self.len].write(value);
        self.len += 1;
        Ok(())
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        unsafe { Some(self.elements[self.len].assume_init_read()) }
    }

    pub fn clear(&mut self) {
        while self.len > 0 {
            self.len -= 1;
            unsafe { self.elements[self.len].assume_init_drop() };
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { self.elements[..self.len].assume_init_ref() }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { self.elements[..self.len].assume_init_mut() }
    }
}

impl<T, const N: usize> Default for FixedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for FixedVec<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N: usize> Deref for FixedVec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for FixedVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> AsRef<[T]> for FixedVec<T, N> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> AsMut<[T]> for FixedVec<T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a FixedVec<T, N> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut FixedVec<T, N> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
