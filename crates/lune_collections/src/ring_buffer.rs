use crate::{CollectionError, CollectionResult};
use std::mem::MaybeUninit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingBufferOverflowPolicy {
    Reject,
    OverwriteOldest,
}

#[derive(Debug)]
pub struct RingBuffer<T, const N: usize> {
    elements: [MaybeUninit<T>; N],
    head: usize,
    len: usize,
    policy: RingBufferOverflowPolicy,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub const fn new(policy: RingBufferOverflowPolicy) -> Self {
        Self {
            elements: [const { MaybeUninit::<T>::uninit() }; N],
            head: 0,
            len: 0,
            policy,
        }
    }

    pub const fn rejecting() -> Self {
        Self::new(RingBufferOverflowPolicy::Reject)
    }

    pub const fn overwriting_oldest() -> Self {
        Self::new(RingBufferOverflowPolicy::OverwriteOldest)
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
    pub fn overflow_policy(&self) -> RingBufferOverflowPolicy {
        self.policy
    }

    pub fn push_back(&mut self, value: T) -> CollectionResult<Option<T>> {
        if N == 0 {
            return Err(CollectionError::RingBufferFull { capacity: N });
        }
        let full = self.is_full();
        if full && self.policy == RingBufferOverflowPolicy::Reject {
            return Err(CollectionError::RingBufferFull { capacity: N });
        }

        let tail = self.tail();

        if full {
            let old = unsafe { self.elements[tail].assume_init_read() };
            self.elements[tail].write(value);
            self.head = Self::wrap(self.head, 1);
            Ok(Some(old))
        } else {
            self.elements[tail].write(value);
            self.len += 1;
            Ok(None)
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let val = unsafe { self.elements[self.head].assume_init_read() };
        self.head = Self::wrap(self.head, 1);
        self.len -= 1;

        Some(val)
    }

    pub fn clear(&mut self) {
        while self.len > 0 {
            self.len -= 1;
            unsafe { self.elements[self.tail()].assume_init_drop() };
        }
    }

    pub fn as_slices(&self) -> (&[T], &[T]) {
        unsafe {
            if self.head + self.len <= N {
                return (
                    self.elements[self.head..(self.head + self.len)].assume_init_ref(),
                    &[],
                );
            }

            let tail = self.head + self.len - N;
            (
                self.elements[self.head..N].assume_init_ref(),
                self.elements[..tail].assume_init_ref(),
            )
        }
    }

    pub fn iter(&self) -> RingBufferIter<'_, T> {
        let (first, second) = self.as_slices();
        RingBufferIter { first, second }
    }

    const fn wrap(start: usize, offset: usize) -> usize {
        wrap(start, offset, N)
    }

    const fn tail(&self) -> usize {
        Self::wrap(self.head, self.len)
    }
}

impl<T, const N: usize> Default for RingBuffer<T, N> {
    fn default() -> Self {
        Self::rejecting()
    }
}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct RingBufferIter<'a, T> {
    first: &'a [T],
    second: &'a [T],
}

impl<'a, T> Iterator for RingBufferIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((elem, rest)) = self.first.split_first() {
            self.first = rest;
            return Some(elem);
        }
        if let Some((elem, rest)) = self.second.split_first() {
            self.second = rest;
            return Some(elem);
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.first.len() + self.second.len();
        (len, Some(len))
    }
}

impl<T> ExactSizeIterator for RingBufferIter<'_, T> {}

impl<'a, T, const N: usize> IntoIterator for &'a RingBuffer<T, N> {
    type Item = &'a T;
    type IntoIter = RingBufferIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

const fn wrap(start: usize, offset: usize, cap: usize) -> usize {
    if cap == 0 {
        return 0;
    }
    (start + offset) % cap
}
