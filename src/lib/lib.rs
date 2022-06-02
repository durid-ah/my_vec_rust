use std::{ptr::{NonNull, self}, marker::PhantomData, mem, alloc::{self, Layout}};
use std::ops::{Deref, DerefMut};

use into_iter::IntoIter;

mod into_iter;

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

   pub fn push(&mut self, elem: T) {
      if self.len == self.cap { self.grow(); }
      unsafe {
         ptr::write(self.ptr.as_ptr().add(self.len), elem);
      }
      self.len += 1; 
   }

   pub fn pop(&mut self) -> Option<T> {
      if self.len == 0 {
         None
      } else {
         self.len -= 1;
         unsafe {
            // Note: Removing an item might call drop on it so using just read prevents it
            Some(ptr::read(self.ptr.as_ptr().add(self.len)))
         }
      }  
   }

   pub fn insert(&mut self, index: usize, elem: T) {
      // Note: `<=` because it's valid to insert after everything
      // which would be equivalent to push.
      assert!(index <= self.len, "index out of bounds");
      if self.cap == self.len { self.grow(); }
  
      unsafe {
         // ptr::copy(src, dest, len): "copy from src to dest len elems"
         ptr::copy(self.ptr.as_ptr().add(index),
                  self.ptr.as_ptr().add(index + 1),
                  self.len - index);
         ptr::write(self.ptr.as_ptr().add(index), elem);
         self.len += 1;
      }
   }

   pub fn remove(&mut self, index: usize) -> T {
      // Note: `<` because it's *not* valid to remove after everything
      assert!(index < self.len, "index out of bounds");
      unsafe {
         self.len -= 1;
         let result = ptr::read(self.ptr.as_ptr().add(index));
         ptr::copy(self.ptr.as_ptr().add(index + 1),
                  self.ptr.as_ptr().add(index),
                  self.len - index);
         result
      }
   }
}

impl<T> Drop for Vec<T> {
   fn drop(&mut self) {
       if self.cap != 0 {
           while let Some(_) = self.pop() { }
           let layout = Layout::array::<T>(self.cap).unwrap();
           unsafe {
               alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
           }
       }
   }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<T> IntoIterator for Vec<T> {
   type Item = T;
   type IntoIter = IntoIter<T>;
   fn into_iter(self) -> IntoIter<T> {
       // Can't destructure Vec since it's Drop
       let ptr = self.ptr;
       let cap = self.cap;
       let len = self.len;

       // Make sure not to drop Vec since that will free the buffer
       mem::forget(self);

       unsafe {
           IntoIter {
               buf: ptr,
               cap,
               start: ptr.as_ptr(),
               end: if cap == 0 {
                   // can't offset off this pointer, it's not allocated!
                   ptr.as_ptr()
               } else {
                   ptr.as_ptr().add(len)
               },
               _marker: PhantomData,
           }
       }
   }
}

