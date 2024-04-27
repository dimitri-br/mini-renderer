use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::{Arc, atomic};
use std::sync::atomic::{AtomicUsize, Ordering};
use wgpu::rwh::{DisplayHandle, HandleError, WindowHandle};
use winit::raw_window_handle::HasWindowHandle;

/// # Handle
///
/// A thread-safe handle to a value
///
/// * `T` - The type of the value
#[derive(Debug)]
pub struct Handle<T>{
    inner: NonNull<RawHandle<T>>,
    phantom: PhantomData<RawHandle<T>>
}

pub struct RawHandle<T>{
    inner: T,
    rc: AtomicUsize,
}

impl<T> Handle<T>{
    /// Create a new Handle with the given value
    ///
    /// * `value` - The value to store in the Handle
    ///
    /// # Returns
    ///
    /// A new Handle with the given value
    pub fn new(value: T) -> Self{
        let raw = Box::new(RawHandle{
            inner: value,
            rc: AtomicUsize::new(1)
        });

        Self{
            inner: NonNull::new(Box::into_raw(raw)).unwrap(),
            phantom: PhantomData
        }
    }
}
unsafe impl<T: Sync + Send> Send for Handle<T> {}
unsafe impl<T: Sync + Send> Sync for Handle<T> {}

impl<T> Deref for Handle<T>{
    type Target = T;

    fn deref(&self) -> &Self::Target{
        unsafe{
            &self.inner.as_ref().inner
        }
    }
}

impl<T> DerefMut for Handle<T>{
    fn deref_mut(&mut self) -> &mut Self::Target{
        unsafe{
            &mut self.inner.as_mut().inner
        }
    }
}

impl<T> Clone for Handle<T>{
    fn clone(&self) -> Self{
        unsafe{
            let rc = self.inner.as_ref().rc.fetch_add(1, Ordering::Relaxed);
            if rc == 0{
                panic!("Handle is already dropped");
            }
        }

        Self{
            inner: self.inner,
            phantom: PhantomData
        }
    }
}

impl<T> Drop for Handle<T>{
    fn drop(&mut self){
        unsafe{
            let rc = self.inner.as_ref().rc.fetch_sub(1, Ordering::Release);
            if rc == 1{
                atomic::fence(Ordering::Acquire);
                let _ = Box::from_raw(self.inner.as_ptr());
            }
        }
    }
}

/* Impl some useful traits for Handle, given the context it's used in rendering */
impl wgpu::rwh::HasWindowHandle for Handle<winit::window::Window>{
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        unsafe { self.inner.as_ref().inner.window_handle() }
    }
}

impl wgpu::rwh::HasDisplayHandle for Handle<winit::window::Window>{
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        unsafe { self.inner.as_ref().inner.display_handle() }
    }
}