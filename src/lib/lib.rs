use std::{ptr, mem};
use std::ops::{Deref, DerefMut};

use into_iter::IntoIter;
use raw_vec::RawVec;

mod into_iter;
mod raw_vec;

pub struct  Vec<T> {
   buf: RawVec<T>,
   len: usize,
}

unsafe impl<T:Send> Send for Vec<T> { }
unsafe impl<T:Sync> Sync for Vec<T> { }

impl<T> Vec<T> {
   pub fn new() -> Self {
      Vec {
         buf: RawVec::new(),
         len: 0,
      }
   }

   fn cap(&self) -> usize {
      self.buf.cap
   }
   
   fn ptr(&self) -> *mut T {
      self.buf.ptr.as_ptr()
   }

   pub fn push(&mut self, elem: T) {
      if self.len == self.cap() {
         self.buf.grow();
      }

      unsafe { ptr::write(self.ptr().add(self.len), elem); }

      // Can't overflow, we'll OOM first.
      self.len += 1;
   }

   pub fn pop(&mut self) -> Option<T> {
      if self.len == 0 {
         None
      } else {
         self.len -= 1;
         unsafe { Some(ptr::read(self.ptr().add(self.len))) }
      }
   }

   pub fn insert(&mut self, index: usize, elem: T) {
      assert!(index <= self.len, "index out of bounds");
      if self.cap() == self.len {
         self.buf.grow();
      }

      unsafe {
         ptr::copy(
            self.ptr().add(index),
            self.ptr().add(index + 1),
            self.len - index,
         );
         ptr::write(self.ptr().add(index), elem);
         self.len += 1;
      }
   }

   pub fn remove(&mut self, index: usize) -> T {
      assert!(index < self.len, "index out of bounds");
      unsafe {
          self.len -= 1;
          let result = ptr::read(self.ptr().add(index));
          ptr::copy(
              self.ptr().add(index + 1),
              self.ptr().add(index),
              self.len - index,
          );
          result
      }
   }
}

impl<T> Drop for Vec<T> {
   fn drop(&mut self) {
      if self.cap() != 0 {
         while let Some(_) = self.pop() { }
      }
   }
}

impl<T> Deref for Vec<T> {
   type Target = [T];
   fn deref(&self) -> &[T] {
      unsafe { std::slice::from_raw_parts(self.ptr(), self.len)}
   }
}

impl<T> DerefMut for Vec<T> {
   fn deref_mut(&mut self) -> &mut [T] {
      unsafe { std::slice::from_raw_parts_mut(self.ptr(), self.len) }
   }
}

impl<T> IntoIterator for Vec<T> {
   type Item = T;
   type IntoIter = IntoIter<T>;
   fn into_iter(self) -> IntoIter<T> {
       unsafe {
           // need to use ptr::read to unsafely move the buf out since it's
           // not Copy, and Vec implements Drop (so we can't destructure it).
           let buf = ptr::read(&self.buf);
           let len = self.len;
           mem::forget(self);

           IntoIter {
               start: buf.ptr.as_ptr(),
               end: if buf.cap == 0 {
                   // can't offset off of a pointer unless it's part of an allocation
                   buf.ptr.as_ptr()
               } else {
                   buf.ptr.as_ptr().add(len)
               },
               _buf: buf,
           }
       }
   }
}

