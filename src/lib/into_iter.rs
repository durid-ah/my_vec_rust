use std::{marker::PhantomData, mem};
use std::ptr::{NonNull, self};
use std::alloc::{self, Layout};

pub struct IntoIter<T> {
   pub buf: NonNull<T>,
   pub cap: usize,
   pub start: *const T,
   pub end: *const T,
   pub _marker: PhantomData<T>,
}

impl<T> Iterator for IntoIter<T> {
   type Item = T;
   fn next(&mut self) -> Option<T> {
      if self.start == self.end {
         None
      } else {
         unsafe {
            let result = ptr::read(self.start);
            self.start = self.start.offset(1);
            Some(result)
         }
      }
   }

   fn size_hint(&self) -> (usize, Option<usize>) {
       let len = (self.end as usize - self.start as usize)
                 / mem::size_of::<T>();
       (len, Some(len))
   }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
   fn next_back(&mut self) -> Option<T> {
      if self.start == self.end {
         None
      } else {
         unsafe {
            self.end = self.end.offset(-1);
            Some(ptr::read(self.end))
         }
      }
   }
}

impl<T> Drop for IntoIter<T> {
   fn drop(&mut self) {
       if self.cap != 0 {
           // drop any remaining elements
           for _ in &mut *self {}
           let layout = Layout::array::<T>(self.cap).unwrap();
           unsafe {
               alloc::dealloc(self.buf.as_ptr() as *mut u8, layout);
           }
       }
   }
}

