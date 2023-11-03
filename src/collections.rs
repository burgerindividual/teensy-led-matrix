use core::mem::MaybeUninit;

pub struct InlineVec<const LEN: usize, T> {
    data: [MaybeUninit<T>; LEN],
    count: usize,
}

impl<const LEN: usize, T> InlineVec<LEN, T> {
    pub fn push(&mut self, value: T) {
        unsafe {
            *self.data.get_mut(self.count).unwrap_unchecked() = MaybeUninit::new(value);
        }
        self.count += 1;
    }

    pub fn clear(&mut self) {
        unsafe {
            for i in 0..self.count {
                self.data.get_mut(i).unwrap_unchecked().assume_init_drop();
            }
        }

        self.count = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// # Safety
    /// The vec must not be empty before calling this function.
    pub fn pop(&mut self) -> T {
        self.count -= 1;
        unsafe {
            self.data
                .get(self.count)
                .unwrap_unchecked()
                .assume_init_read()
        }
    }

    pub fn get_slice(&self) -> &[T] {
        // SAFETY: count shouldn't ever be able to be incremented past LEN, and the contents should
        // be initialized
        unsafe { MaybeUninit::slice_assume_init_ref(self.data.get_unchecked(0..(self.count))) }
    }

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
        *self
    }
}

impl<const LEN: usize, T: Copy> Copy for InlineVec<LEN, T> {}
