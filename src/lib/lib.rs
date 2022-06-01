use std::{ptr::NonNull, marker::PhantomData, mem};

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

    
}
