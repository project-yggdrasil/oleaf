use std::{mem, slice};

/// An ABI-compatible `std::vector` that is borrowed from the C++ side.
///
/// This represents a contiguous storage of elements with pointers to
/// the boundaries. The lifetime duration of said storage is fully managed
/// by C++ to which an immutable view on the Rust side is granted.
///
/// # Safety
///
/// The user must ensure that their handle to a [`Vector`] instance does
/// not outlive the corresponding object on the C++ side.
#[repr(C)]
pub struct Vector<T> {
    head: *mut T,
    tail: *mut T,
    end: *mut T,
}

impl<T> Vector<T> {
    /// Gets the length of the vector measured by the elements it holds.
    ///
    /// This is done by calculating the offset between the head and tail
    /// pointers of the allocation region divided by `std::mem::size_of::<T>()`.
    ///
    /// # Panics
    ///
    /// This function panics when `T` is a Zero-Sized Type ("ZST") or when
    /// its size exceeds [`isize::MAX`] which is the largest size a Rust
    /// type may have.
    ///
    /// Likewise, the distance of the insertion region may not exceed
    /// [`isize::MAX`].
    pub fn len(&self) -> usize {
        let pointee_size = mem::size_of::<T>();
        let region = self.tail as usize - self.head as usize;

        assert!(
            0 < pointee_size
                && pointee_size <= isize::MAX as usize
                && region <= isize::MAX as usize
        );
        region / pointee_size
    }

    /// Checks if the vector is empty, i.e. is holding zero elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the capacity that is reserved for the storage of this vector.
    ///
    /// This is done by calculating the size of the allocated region divided
    /// by `std::mem::size_of::<T>()`.
    ///
    /// # Panics
    ///
    /// This function panics when `T` is a Zero-Sized Type ("ZST") or when
    /// its size exceeds [`isize::MAX`] which is the largest size a Rust
    /// type may have.
    ///
    /// Likewise, the distance of the allocation region may not exceed
    /// [`isize::MAX`].
    pub fn capacity(&self) -> usize {
        let pointee_size = mem::size_of::<T>();
        let region = self.end as usize - self.head as usize;

        assert!(
            0 < pointee_size
                && pointee_size <= isize::MAX as usize
                && region <= isize::MAX as usize
        );
        region / pointee_size
    }

    /// Gets the underlying vector storage as a contiguous Rust slice.
    ///
    /// # Panics
    ///
    /// This function panics under the same conditions as [`Vector::len`].
    ///
    /// # Safety
    ///
    /// The caller must ensure that the inferred lifetime does not exceed
    /// the duration of the managed object on the C++ side.
    pub unsafe fn as_slice(&self) -> &[T] {
        if self.head.is_null() {
            // Constructing a slice at address 0 is undefined behavior.
            // We will use a filler value here.
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.head, self.len()) }
        }
    }

    /// Gets a raw pointer to the start of the insertion region.
    ///
    /// The caller must ensure that the pointer does not outlive the vector
    /// on the C++ side. Further, the vector have its previous allocation
    /// region invalidated when a reallocation on the C++ side is triggered.
    ///
    /// The caller must also ensure that the pointed-to memory is **never**
    /// written to.
    pub fn as_ptr(&self) -> *const T {
        self.head
    }
}
