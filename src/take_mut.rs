// Copyright (c) 2023 alecmocatta
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
//
// Code snippet from the replace_with crate, adapted for usage
// in this environment.

use core::mem::ManuallyDrop;
use core::ptr;

use teensy4_panic::sos;

struct OnDrop<F: FnOnce()>(ManuallyDrop<F>);
impl<F: FnOnce()> Drop for OnDrop<F> {
    #[inline(always)]
    fn drop(&mut self) {
        (unsafe { ptr::read(&*self.0) })();
    }
}

#[inline(always)]
pub fn take_mut<T, F: FnOnce(T) -> T>(dest: &mut T, f: F) {
    unsafe {
        let old = ptr::read(dest);
        // unwrap detector?
        let new = {
            let x = OnDrop(ManuallyDrop::new(|| sos()));
            let t = f(old);
            let mut x = ManuallyDrop::new(x);
            ManuallyDrop::drop(&mut x.0);
            t
        };
        ptr::write(dest, new);
    }
}
