use bincode::enc::Encoder;
use bincode::error::EncodeError;
use core::mem::{ManuallyDrop, MaybeUninit};
use core::ops::Index;
use core::{mem, ptr, slice};

#[derive(Debug)]
pub struct NoVec<T, const N: usize> {
    inner: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> NoVec<T, N> {
    pub fn new() -> Self {
        Self {
            inner: core::array::from_fn(|_| MaybeUninit::<T>::uninit()),
            len: 0,
        }
    }

    pub fn push(&mut self, el: T) {
        // Check capacity
        if self.len >= N {
            panic!("")
        }

        self.inner[self.len].write(el);
        self.len += 1;
    }

    pub fn append<const M: usize>(&mut self, other: [T; M]) {
        if self.len + M > N {
            panic!("");
        }

        // bit-copy array to our array
        // We use ManuallyDrop in order not to free memory twice
        let other: ManuallyDrop<[_; M]> = ManuallyDrop::new(other);
        let dst = unsafe {
            mem::transmute::<*mut MaybeUninit<T>, *mut T>(self.inner.as_mut_ptr().add(self.len))
        };
        unsafe { ptr::copy_nonoverlapping(other.as_ptr(), dst, M) };

        // Increase
        self.len += M;
    }

    pub fn iter(&self) -> slice::Iter<T> {
        self.as_slice().iter()
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns false.
    /// This method operates in place and preserves the order of the retained
    /// elements.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        // Check the implementation of
        // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.retain
        // for safety arguments (especially regarding panics in f and when
        // dropping elements). Implementation closely mirrored here.

        let original_len = self.len;
        self.len = 0;

        struct BackshiftOnDrop<'a, T, const CAP: usize> {
            v: &'a mut NoVec<T, CAP>,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<T, const CAP: usize> Drop for BackshiftOnDrop<'_, T, CAP> {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    unsafe {
                        ptr::copy(
                            self.v.as_ptr().add(self.processed_len),
                            self.v
                                .as_mut_ptr()
                                .add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                    }
                }
                self.v.len = self.original_len - self.deleted_cnt;
            }
        }

        let mut g = BackshiftOnDrop {
            v: self,
            processed_len: 0,
            deleted_cnt: 0,
            original_len,
        };

        #[inline(always)]
        fn process_one<F: FnMut(&mut T) -> bool, T, const CAP: usize, const DELETED: bool>(
            f: &mut F,
            g: &mut BackshiftOnDrop<'_, T, CAP>,
        ) -> bool {
            let cur = unsafe { g.v.as_mut_ptr().add(g.processed_len) };
            if !f(unsafe { &mut *cur }) {
                g.processed_len += 1;
                g.deleted_cnt += 1;
                unsafe { ptr::drop_in_place(cur) };
                return false;
            }
            if DELETED {
                unsafe {
                    let hole_slot = cur.sub(g.deleted_cnt);
                    ptr::copy_nonoverlapping(cur, hole_slot, 1);
                }
            }
            g.processed_len += 1;
            true
        }

        // Stage 1: Nothing was deleted.
        while g.processed_len != original_len {
            if !process_one::<F, T, N, false>(&mut f, &mut g) {
                break;
            }
        }

        // Stage 2: Some elements were deleted.
        while g.processed_len != original_len {
            process_one::<F, T, N, true>(&mut f, &mut g);
        }

        drop(g);
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: `slice::from_raw_parts` requires pointee is a contiguous, aligned buffer of size
        // `len` containing properly-initialized `T`s. Data must not be mutated for the returned
        // lifetime. Further, `len * mem::size_of::<T>` <= `ISIZE::MAX`, and allocation does not
        // "wrap" through overflowing memory addresses.
        //
        // * Vec API guarantees that self.buf:
        //      * contains only properly-initialized items within 0..len
        //      * is aligned, contiguous, and valid for `len` reads
        //      * obeys size and address-wrapping constraints
        let inited = self.inner.as_ptr() as *const T;
        unsafe { slice::from_raw_parts(inited, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let inited = self.inner.as_mut_ptr() as *mut T;
        unsafe { slice::from_raw_parts_mut(inited, self.len) }
    }

    pub unsafe fn as_ptr(&self) -> *const T {
        self.inner.as_ptr() as _
    }

    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_mut_ptr() as _
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T: Clone, const N: usize> NoVec<T, N> {
    pub fn append_slice(&mut self, other: &[T]) {
        if self.len + other.len() > N {
            panic!("");
        }

        for (uninit, el) in self.inner[self.len..].iter_mut().zip(other.iter()) {
            uninit.write(el.clone());
        }

        self.len += other.len();
    }

    pub fn append_slice_shadowed<'a, 'new>(&'a mut self, other: &'new [T])
    where
        'a: 'new,
    {
        if self.len + other.len() > N {
            panic!("");
        }
    }
}

impl<T: Ord, const N: usize> NoVec<T, N> {
    pub fn sort(&mut self) {
        self.as_mut_slice().sort();
    }
}

impl<T: PartialEq, const N: usize> NoVec<T, N> {
    pub fn contains(&self, x: &T) -> bool {
        self.as_slice().contains(x)
    }
}

impl<T, const N: usize> Default for NoVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, const N: usize> Clone for NoVec<T, N> {
    fn clone(&self) -> Self {
        let mut this = Self::new();
        this.append_slice(self.as_slice());
        this
    }
}

pub struct IntoIter<T, const N: usize> {
    cur: usize,
    inner: NoVec<T, N>,
}

impl<T, const N: usize> IntoIter<T, N> {
    pub fn new(inner: NoVec<T, N>) -> Self {
        Self { cur: 0, inner }
    }
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.inner.len {
            None
        } else {
            let index = self.cur;
            self.cur += 1;
            unsafe { Some(ptr::read(self.inner.as_ptr().add(index))) }
        }
    }
}

impl<T, const N: usize> Drop for IntoIter<T, N> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(slice::from_raw_parts_mut(
                self.inner.as_mut_ptr().add(self.cur),
                self.inner.len - self.cur,
            ));
        }
    }
}

impl<T, const N: usize> FromIterator<T> for NoVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut this = Self::new();
        for el in iter {
            this.push(el);
        }

        this
    }
}

impl<T, const N: usize> Extend<T> for NoVec<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for el in iter {
            self.push(el);
        }
    }
}

impl<T, const N: usize> IntoIterator for NoVec<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<T, const N: usize> Index<usize> for NoVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!("")
        }

        unsafe { self.inner[index].assume_init_ref() }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a NoVec<T, N> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a, T, const N: usize> {
    cur: usize,
    inner: &'a NoVec<T, N>,
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.len == self.cur {
            None
        } else {
            let index = self.cur;
            self.cur += 1;
            Some(&self.inner[index])
        }
    }
}

impl<T, const N: usize> Drop for NoVec<T, N> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.as_mut_slice());
        }
    }
}

impl<T: serde::Serialize, const N: usize> serde::Serialize for NoVec<T, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(self.as_slice(), serializer)
    }
}

impl<T: bincode::Encode, const N: usize> bincode::Encode for NoVec<T, N> {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        bincode::Encode::encode(self.as_slice(), encoder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_len() {
        let mut v: NoVec<i32, 5> = NoVec::new();

        v.push(1);
        assert_eq!(v.len(), 1);
        assert!(!v.is_empty());

        v.push(2);
        v.push(3);
        assert_eq!(v.len(), 3);

        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
        assert_eq!(v[2], 3);
    }

    #[test]
    #[should_panic]
    fn test_push_overflow() {
        let mut v: NoVec<i32, 2> = NoVec::new();
        v.push(1);
        v.push(2);
        v.push(3); // should panic
    }

    #[test]
    fn test_append_fixed_array() {
        let mut v: NoVec<i32, 10> = NoVec::new();
        v.push(0);
        v.append([1, 2, 3]);

        assert_eq!(v.len(), 4);
        assert_eq!(v.as_slice(), &[0, 1, 2, 3]);
    }

    #[test]
    #[should_panic]
    fn test_append_overflow() {
        let mut v: NoVec<i32, 3> = NoVec::new();
        v.push(0);
        v.append([1, 2, 3]);
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds() {
        let mut v: NoVec<i32, 5> = NoVec::new();
        v.push(1);
        let _ = v[1]; // should panic
    }

    #[test]
    fn test_into_iter() {
        let mut v: NoVec<i32, 5> = NoVec::new();
        v.push(10);
        v.push(20);
        v.push(30);

        let mut iter = v.into_iter();
        assert_eq!(iter.next(), Some(10));
        assert_eq!(iter.next(), Some(20));
        assert_eq!(iter.next(), Some(30));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_extend() {
        let mut v: NoVec<i32, 10> = NoVec::new();
        v.push(0);
        v.extend([1, 2, 3]);

        assert_eq!(v.len(), 4);
        assert_eq!(v.as_slice(), &[0, 1, 2, 3]);
    }

    #[test]
    fn test_retain_filter_evens() {
        let mut v: NoVec<i32, 10> = NoVec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(4);
        v.push(5);

        v.retain(|x| *x % 2 == 0);
        assert_eq!(v.as_slice(), &[2, 4]);
    }

    #[test]
    fn test_zero_capacity_iter() {
        let v: NoVec<i32, 0> = NoVec::new();
        let mut iter = v.iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_zero_capacity_retain() {
        let mut v: NoVec<i32, 0> = NoVec::new();
        v.retain(|_| true);
        assert!(v.is_empty());
    }

    #[test]
    fn test_zero_capacity_extend_empty() {
        let mut v: NoVec<i32, 0> = NoVec::new();
        v.extend(core::iter::empty());
        assert!(v.is_empty());
    }

    #[test]
    fn test_empty_append_empty_slice() {
        let mut v: NoVec<i32, 10> = NoVec::new();
        v.append_slice(&[]);
        assert!(v.is_empty());
    }
}
