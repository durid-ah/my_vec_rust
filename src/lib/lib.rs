use std::{ptr::NonNull, marker::PhantomData, mem, alloc::{self, Layout}};

pub struct  Vec<T> {
   ptr: NonNull<T>,
   cap: usize,
   len: usize,
   _marker: PhantomData<T>
}

unsafe impl<T:Send> Send for Vec<T> { }
unsafe impl<T:Sync> Sync for Vec<T> { }

impl<T> Vec<T> {
   pub fn new() -> Self {
      assert!(mem::size_of::<T>() != 0, "Unable to handle ZSTs");
      Vec { ptr: NonNull::dangling(), cap: 0, len: 0, _marker: PhantomData }
   }

    fn grow(&mut self) {
      let (new_cap, new_layout) = if self.cap == 0 {
         (1, Layout::array::<T>(1).unwrap())
      } else {
         // This can't overflow since self.cap <= isize::MAX
         let new_cap = 2 * self.cap;

         // `Layout::array` checks that the number of bytes is <= usize::MAX,
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
    }
}
