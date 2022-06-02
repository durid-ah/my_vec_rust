use std::{ptr::NonNull, marker::PhantomData, mem, alloc::{Layout, self}};


pub(super) struct RawVec<T> {
   pub(super) ptr: NonNull<T>,
   pub(super) cap: usize,
   _marker: PhantomData<T>,
}

unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

impl<T> RawVec<T> {
   pub(super) fn new() -> Self {
       assert!(mem::size_of::<T>() != 0, "TODO: implement ZST support");
       RawVec {
           ptr: NonNull::dangling(),
           cap: 0,
           _marker: PhantomData,
       }
   }

   pub(super) fn grow(&mut self) {
       let (new_cap, new_layout) = if self.cap == 0 {
           (1, Layout::array::<T>(1).unwrap())
       } else {
           // This can't overflow because we ensure self.cap <= isize::MAX.
           let new_cap = 2 * self.cap;

           // Layout::array checks that the number of bytes is <= usize::MAX,
           // but this is redundant since old_layout.size() <= isize::MAX,
           // so the `unwrap` should never fail.
           let new_layout = Layout::array::<T>(new_cap).unwrap();
           (new_cap, new_layout)
       };

       // Ensure that the new allocation doesn't exceed `isize::MAX` bytes.
       assert!(new_layout.size() <= isize::MAX as usize, "Allocation too large");

       let new_ptr = if self.cap == 0 {
           unsafe { alloc::alloc(new_layout) }
       } else {
           let old_layout = Layout::array::<T>(self.cap).unwrap();
           let old_ptr = self.ptr.as_ptr() as *mut u8;
           unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
       };

       // If allocation fails, `new_ptr` will be null, in which case we abort.
       self.ptr = match NonNull::new(new_ptr as *mut T) {
           Some(p) => p,
           None => alloc::handle_alloc_error(new_layout),
       };
       self.cap = new_cap;
   }
}

impl<T> Drop for RawVec<T> {
   fn drop(&mut self) {
      if self.cap != 0 {
         let layout = Layout::array::<T>(self.cap).unwrap();
         unsafe { alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout); }
      }
   }
}
