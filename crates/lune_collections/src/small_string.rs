use crate::{CollectionError, CollectionResult};
use std::alloc::{Layout, alloc};
use std::fmt;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};
use std::slice;

#[derive(Debug)]
enum SmallStringData<const N: usize> {
    Inline([MaybeUninit<u8>; N]),
    Heap { ptr: NonNull<u8>, cap: usize },
}

#[derive(Debug)]
pub struct SmallString<const N: usize> {
    bytes: SmallStringData<N>,
    len: usize,
}

impl<const N: usize> SmallString<N> {
    pub const fn new() -> Self {
        Self {
            bytes: SmallStringData::Inline([const { MaybeUninit::<u8>::uninit() }; N]),
            len: 0,
        }
    }

    pub const fn inline_capacity(&self) -> usize {
        N
    }

    pub fn capacity(&self) -> usize {
        match self.bytes {
            SmallStringData::Inline(_) => self.inline_capacity(),
            SmallStringData::Heap { cap, .. } => cap,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_spilled(&self) -> bool {
        matches!(self.bytes, SmallStringData::Heap { .. })
    }

    pub fn push_str(&mut self, text: &str) -> CollectionResult<()> {
        if text.is_empty() {
            return Ok(());
        }

        let added = text.len();
        let new_len = self.len + added;

        match &mut self.bytes {
            SmallStringData::Inline(inline) => {
                if new_len <= N {
                    unsafe {
                        ptr::copy_nonoverlapping(
                            text.as_ptr(),
                            inline.as_mut_ptr().add(self.len) as *mut u8,
                            added,
                        );
                    }
                    self.len = new_len;
                } else {
                    let new_cap = (N.max(1) * 2).max(new_len);
                    let layout = Layout::array::<u8>(new_cap).map_err(|_| {
                        CollectionError::LayoutOverflow {
                            capacity: new_cap as isize,
                        }
                    })?;
                    let new_ptr = unsafe { alloc(layout) };
                    let new_ptr =
                        NonNull::new(new_ptr).ok_or(CollectionError::AllocationError { layout })?;
                    unsafe {
                        ptr::copy_nonoverlapping(
                            inline.as_ptr() as *const u8,
                            new_ptr.as_ptr(),
                            self.len,
                        );
                        ptr::copy_nonoverlapping(
                            text.as_ptr(),
                            new_ptr.as_ptr().add(self.len),
                            added,
                        );
                    }
                    self.bytes = SmallStringData::Heap {
                        ptr: new_ptr,
                        cap: new_cap,
                    };
                    self.len = new_len;
                }
            }
            SmallStringData::Heap { ptr, cap } => {
                if new_len <= *cap {
                    unsafe {
                        ptr::copy_nonoverlapping(text.as_ptr(), ptr.as_ptr().add(self.len), added);
                    }
                    self.len = new_len;
                } else {
                    let new_cap = (*cap * 2).max(new_len);
                    let layout = Layout::array::<u8>(new_cap).map_err(|_| {
                        CollectionError::LayoutOverflow {
                            capacity: new_cap as isize,
                        }
                    })?;
                    let old_layout =
                        Layout::array::<u8>(*cap).expect("layout was valid at allocation time");
                    let new_ptr = unsafe { alloc(layout) };
                    let new_ptr =
                        NonNull::new(new_ptr).ok_or(CollectionError::AllocationError { layout })?;
                    unsafe {
                        ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), self.len);
                        ptr::copy_nonoverlapping(
                            text.as_ptr(),
                            new_ptr.as_ptr().add(self.len),
                            added,
                        );
                        std::alloc::dealloc(ptr.as_ptr(), old_layout);
                    }
                    *ptr = new_ptr;
                    *cap = new_cap;
                    self.len = new_len;
                }
            }
        }

        Ok(())
    }

    pub fn push(&mut self, value: char) -> CollectionResult<()> {
        let mut buf = [0u8; 4];
        let encoded = value.encode_utf8(&mut buf);
        self.push_str(encoded)
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn as_str(&self) -> &str {
        let bytes = match &self.bytes {
            SmallStringData::Inline(inline) => unsafe {
                slice::from_raw_parts(inline.as_ptr() as *const u8, self.len)
            },
            SmallStringData::Heap { ptr, .. } => unsafe {
                slice::from_raw_parts(ptr.as_ptr(), self.len)
            },
        };
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    pub fn as_mut_str(&mut self) -> &mut str {
        let len = self.len;
        let bytes = match &mut self.bytes {
            SmallStringData::Inline(inline) => unsafe {
                slice::from_raw_parts_mut(inline.as_mut_ptr() as *mut u8, len)
            },
            SmallStringData::Heap { ptr, .. } => unsafe {
                slice::from_raw_parts_mut(ptr.as_ptr(), len)
            },
        };
        unsafe { std::str::from_utf8_unchecked_mut(bytes) }
    }
}

impl<const N: usize> Default for SmallString<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Drop for SmallString<N> {
    fn drop(&mut self) {
        if let SmallStringData::Heap { ptr, cap } = self.bytes {
            let layout = Layout::array::<u8>(cap).expect("layout was valid at allocation time");
            unsafe { std::alloc::dealloc(ptr.as_ptr(), layout) };
        }
    }
}

impl<const N: usize> fmt::Write for SmallString<N> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.push_str(text).map_err(|_| fmt::Error)
    }

    fn write_char(&mut self, value: char) -> fmt::Result {
        self.push(value).map_err(|_| fmt::Error)
    }
}

impl<const N: usize> Deref for SmallString<N> {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> DerefMut for SmallString<N> {
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl<const N: usize> AsRef<str> for SmallString<N> {
    fn as_ref(&self) -> &str {
        self
    }
}

impl<const N: usize> AsMut<str> for SmallString<N> {
    fn as_mut(&mut self) -> &mut str {
        self
    }
}
