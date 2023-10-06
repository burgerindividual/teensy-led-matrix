use core::mem::MaybeUninit;

pub struct InlineVec<const LEN: usize, T> {
    data: [MaybeUninit<T>; LEN],
    count: usize,
}

impl<const LEN: usize, T> InlineVec<LEN, T> {
    #[inline(always)]
    pub fn push(&mut self, value: T) {
        self.data[self.count] = MaybeUninit::new(value);
        self.count += 1;
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        unsafe {
            for i in 0..self.count {
                self.data[i].assume_init_drop();
            }
        }

        self.count = 0;
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// # Safety
    /// The vec must not be empty before calling this function.
    #[inline(always)]
    pub unsafe fn pop(&mut self) -> T {
        self.count -= 1;
        self.data.get_unchecked(self.count).assume_init_read()
    }

    #[inline(always)]
    pub fn get_slice(&self) -> &[T] {
        // SAFETY: count shouldn't ever be able to be incremented past LEN, and the contents should
        // be initialized
        unsafe { MaybeUninit::slice_assume_init_ref(self.data.get_unchecked(0..(self.count))) }
    }

    #[inline(always)]
    pub fn get_slice_mut(&mut self) -> &mut [T] {
        // SAFETY: count shouldn't ever be able to be incremented past LEN, and the contents should
        // be initialized
        unsafe { MaybeUninit::slice_assume_init_mut(self.data.get_unchecked_mut(0..(self.count))) }
    }
}

impl<const LEN: usize, T> Default for InlineVec<LEN, T> {
    fn default() -> Self {
        Self {
            data: unsafe { MaybeUninit::<[MaybeUninit<T>; LEN]>::uninit().assume_init() },
            count: 0,
        }
    }
}

impl<const LEN: usize, T: Copy> Clone for InlineVec<LEN, T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            count: self.count,
        }
    }
}

impl<const LEN: usize, T: Copy> Copy for InlineVec<LEN, T> {}
