use crate::{CollectionError, CollectionResult};
use std::alloc::{Layout, alloc};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};
use std::slice;

#[derive(Debug)]
enum SmallVecData<T, const N: usize> {
    Inline([MaybeUninit<T>; N]),
    Heap { ptr: NonNull<T>, cap: usize },
}

#[derive(Debug)]
pub struct SmallVec<T, const N: usize> {
    elements: SmallVecData<T, N>,
    len: usize,
}

impl<T, const N: usize> SmallVec<T, N> {
    const IS_ZST: bool = size_of::<T>() == 0;

    pub const fn new() -> Self {
        Self {
            elements: SmallVecData::Inline([const { MaybeUninit::<T>::uninit() }; N]),
            len: 0,
        }
    }

    pub const fn inline_capacity(&self) -> usize {
        N
    }

    pub fn capacity(&self) -> usize {
        match self.elements {
            SmallVecData::Inline(_) => self.inline_capacity(),
            SmallVecData::Heap { cap, .. } => cap,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_spilled(&self) -> bool {
        matches!(self.elements, SmallVecData::Heap { .. })
    }

    pub fn push(&mut self, value: T) -> CollectionResult<()> {
        match &mut self.elements {
            SmallVecData::Inline(inline) => match self.len == N {
                true => {
                    let (new_ptr, new_cap) = if Self::IS_ZST {
                        (NonNull::dangling(), usize::MAX)
                    } else {
                        let new_cap = N.max(1) * 2;
                        let layout = Layout::array::<T>(new_cap).map_err(|_| {
                            CollectionError::LayoutOverflow {
                                capacity: new_cap as isize,
                            }
                        })?;
                        let new_ptr = unsafe { alloc(layout) as *mut T };
                        let new_ptr = NonNull::new(new_ptr)
                            .ok_or(CollectionError::AllocationError { layout })?;
                        (new_ptr, new_cap)
                    };

                    unsafe {
                        ptr::copy_nonoverlapping(
                            inline.as_ptr() as *const T,
                            new_ptr.as_ptr(),
                            self.len,
                        );
                        ptr::write(new_ptr.as_ptr().add(self.len), value);
                    };
                    self.len += 1;
                    self.elements = SmallVecData::Heap {
                        ptr: new_ptr,
                        cap: new_cap,
                    };
                }
                false => {
                    inline[self.len].write(value);
                    self.len += 1;
                }
            },
            SmallVecData::Heap { cap, ptr } => match !Self::IS_ZST && *cap == self.len {
                true => {
                    let new_cap = *cap * 2;
                    let layout = Layout::array::<T>(new_cap).map_err(|_| {
                        CollectionError::LayoutOverflow {
                            capacity: new_cap as isize,
                        }
                    })?;
                    let old_layout =
                        Layout::array::<T>(*cap).expect("layout was valid at allocation time");
                    unsafe {
                        let new_ptr = alloc(layout) as *mut T;
                        let new_ptr = NonNull::new(new_ptr)
                            .ok_or(CollectionError::AllocationError { layout })?;
                        ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), self.len);
                        std::alloc::dealloc(ptr.as_ptr() as *mut u8, old_layout);
                        *ptr = new_ptr;
                        ptr::write(ptr.as_ptr().add(self.len), value);
                    };
                    self.len += 1;
                    *cap = new_cap;
                }
                false => {
                    unsafe { ptr::write(ptr.as_ptr().add(self.len), value) };
                    self.len += 1;
                }
            },
        }

        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.len -= 1;
        match &mut self.elements {
            SmallVecData::Inline(inline) => Some(unsafe { inline[self.len].assume_init_read() }),
            SmallVecData::Heap { ptr, .. } => {
                Some(unsafe { ptr::read(ptr.as_ptr().add(self.len)) })
            }
        }
    }

    pub fn clear(&mut self) {
        match &mut self.elements {
            SmallVecData::Inline(inline) => {
                while self.len > 0 {
                    self.len -= 1;
                    unsafe { inline[self.len].assume_init_drop() };
                }
            }
            SmallVecData::Heap { ptr, .. } => {
                while self.len > 0 {
                    self.len -= 1;
                    unsafe {
                        ptr::drop_in_place(ptr.as_ptr().add(self.len));
                    };
                }
            }
        }
    }

    pub fn as_slice(&self) -> &[T] {
        match self.elements {
            SmallVecData::Inline(ref inline) => unsafe { inline[..self.len].assume_init_ref() },
            SmallVecData::Heap { ptr, .. } => unsafe {
                std::slice::from_raw_parts(ptr.as_ptr(), self.len)
            },
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match &mut self.elements {
            SmallVecData::Inline(inline) => unsafe { inline[..self.len].assume_init_mut() },
            SmallVecData::Heap { ptr, .. } => unsafe {
                std::slice::from_raw_parts_mut(ptr.as_ptr(), self.len)
            },
        }
    }
}

impl<T, const N: usize> Default for SmallVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Drop for SmallVec<T, N> {
    fn drop(&mut self) {
        self.clear();
        if !Self::IS_ZST
            && let SmallVecData::Heap { ptr, cap } = self.elements
        {
            let layout = Layout::array::<T>(cap).expect("layout was valid at allocation time");
            unsafe { std::alloc::dealloc(ptr.as_ptr() as *mut u8, layout) };
        }
    }
}

impl<T, const N: usize> Deref for SmallVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for SmallVec<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> AsRef<[T]> for SmallVec<T, N> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> AsMut<[T]> for SmallVec<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a SmallVec<T, N> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut SmallVec<T, N> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
