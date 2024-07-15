#![feature(const_maybe_uninit_as_mut_ptr)]
#![feature(const_mut_refs)]
#![feature(const_ptr_write)]
#![feature(const_trait_impl)]
#![feature(effects)]
#![feature(ptr_metadata)]
#![feature(unsize)]

#![allow(incomplete_features)]

#![no_std]

use const_default::ConstDefault;
use core::borrow::{Borrow, BorrowMut};
use core::fmt::{self, Debug, Display, Formatter};
use core::marker::{PhantomData, Unsize};
use core::mem::{ManuallyDrop, MaybeUninit, align_of, size_of};
use core::ops::{Deref, DerefMut};
use core::ptr::{self, Pointee};

/// Stack-allocated space.
///
/// # Safety
///
/// This trait can be implemented only through unconditional delegating to another implementation.
#[const_trait]
pub unsafe trait Buf: ConstDefault {
    /// Allocated space location as an immutable pointer.
    fn as_ptr(&self) -> *const u8;

    /// Allocated space location as a mutable pointer.
    fn as_mut_ptr(&mut self) -> *mut u8;

    /// Allocated space alignment.
    fn align() -> usize;

    /// Allocated space size.
    fn len() -> usize;
}

/// Stack-allocated space appropriated to store the specific type.
///
/// Appropriated for emplacing types with
/// size less or equal `size_of::<T>()` and alignment less or equal `align_of::<T>()`.
pub struct BufFor<T>(MaybeUninit<T>);

impl<T> ConstDefault for BufFor<T> {
    const DEFAULT: Self = BufFor(MaybeUninit::uninit());
}

unsafe impl<T> const Buf for BufFor<T> {
    fn as_ptr(&self) -> *const u8 { self.0.as_ptr() as _ }

    fn as_mut_ptr(&mut self) -> *mut u8 { self.0.as_mut_ptr() as _ }

    fn align() -> usize { align_of::<T>() }

    fn len() -> usize { size_of::<T>() }
}

/// A helper type for creating [`BufFor`] any of two types.
///
/// The type satisfies the following properties:
///
/// 1. `size_of::<AnyOf2<T1, T2>>() >= size_of::<T1>()`
/// 2. `size_of::<AnyOf2<T1, T2>>() >= size_of::<T2>()`
/// 3. `align_of::<AnyOf2<T1, T2>>() >= align_of::<T1>()`
/// 4. `align_of::<AnyOf2<T1, T2>>() >= align_of::<T2>()`
#[repr(C)]
pub union AnyOf2<T1, T2> {
    _a: ManuallyDrop<T1>,
    _b: ManuallyDrop<T2>,
}

/// A stack-allocated container that can store dynamically sized types.
pub struct ArrayBox<'a, T: ?Sized + 'a, B: Buf> {
    buf: B,
    metadata: <T as Pointee>::Metadata,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: ?Sized + 'a, B: Buf> Drop for ArrayBox<'a, T, B> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.as_mut_ptr()) };
    }
}

impl<'a, T: ?Sized + 'a, B: Buf> ArrayBox<'a, T, B> {
    /// Allocates memory on stack and then places `source` into it as DST `T`.
    pub const fn new<S: Unsize<T>>(mut source: S) -> Self where B: ~const Buf + ConstDefault {
        assert!(B::align() >= align_of::<S>());
        assert!(B::len() >= size_of::<S>());
        let metadata = (&mut source as *mut T).to_raw_parts().1;
        let mut res = ArrayBox { buf: B::DEFAULT, metadata, phantom: PhantomData };
        unsafe { ptr::write::<S>(res.buf.as_mut_ptr() as *mut S, source) };
        res
    }

    /// Return raw immutable pointer to the stored object.
    pub fn as_ptr(&self) -> *const T {
        let metadata = self.metadata;
        ptr::from_raw_parts(self.buf.as_ptr() as *const (), metadata)
    }

    /// Return raw mutable pointer to the stored object.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        let metadata = self.metadata;
        ptr::from_raw_parts_mut(self.buf.as_mut_ptr() as *mut (), metadata)
    }
}

impl<'a, T: ?Sized + 'a, B: Buf> AsRef<T> for ArrayBox<'a, T, B> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }
}

impl<'a, T: ?Sized + 'a, B: Buf> AsMut<T> for ArrayBox<'a, T, B> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.as_mut_ptr() }
    }
}

impl<'a, T: ?Sized + 'a, B: Buf> Borrow<T> for ArrayBox<'a, T, B> {
    fn borrow(&self) -> &T { self.as_ref() }
}

impl<'a, T: ?Sized + 'a, B: Buf> BorrowMut<T> for ArrayBox<'a, T, B> {
    fn borrow_mut(&mut self) -> &mut T { self.as_mut() }
}

impl<'a, T: ?Sized + 'a, B: Buf> Deref for ArrayBox<'a, T, B> {
    type Target = T;

    fn deref(&self) -> &T { self.as_ref() }
}

impl<'a, T: ?Sized + 'a, B: Buf> DerefMut for ArrayBox<'a, T, B> {
    fn deref_mut(&mut self) -> &mut T { self.as_mut() }
}

impl<'a, T: Debug + ?Sized + 'a, B: Buf> Debug for ArrayBox<'a, T, B> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl<'a, T: Display + ?Sized + 'a, B: Buf> Display for ArrayBox<'a, T, B> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}
