use std::{mem, ptr};
use std::alloc::{self, Layout};

use crate::raw_vec::RawVec;

pub struct IntoIter<T> {
   pub(super) _buf: RawVec<T>, // we don't actually care about this. Just need it to live.
   pub(super) start: *const T,
   pub(super) end: *const T,
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
       // only need to ensure all our elements are read;
       // buffer will clean itself up afterwards.
       for _ in &mut *self {}
   }
}

