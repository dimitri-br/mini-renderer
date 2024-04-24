use std::sync::{Arc, Mutex, MutexGuard};

/// # MutHandle
///
/// A thread-safe mutable handle to a value
///
/// * `T` - The type of the value
pub struct MutHandle<T>{
    inner: Arc<Mutex<T>>,
}

impl<T> MutHandle<T>{
    /// Create a new MutHandle with the given value
    ///
    /// * `value` - The value to store in the MutHandle
    ///
    /// # Returns
    ///
    /// A new MutHandle with the given value
    pub fn new(value: T) -> Self{
        Self{
            inner: Arc::new(Mutex::new(value))
        }
    }

    /// # Get
    ///
    /// Get a reference to the value stored in the MutHandle
    ///
    /// # Returns
    ///
    /// A reference to the value stored in the MutHandle
    pub fn get(&self) -> MutexGuard<T>{
        self.inner.lock().unwrap()
    }
    
    pub fn get_inner(&self) -> Arc<Mutex<T>>{
        self.inner.clone()
    }
    
    /// # Get with lifetime
    /// 
    /// Get a reference to the value stored in the MutHandle with a lifetime
    pub fn get_ref<'a, 'b>(&'a self) -> MutexGuard<'b, T> where 'a: 'b{
        self.inner.lock().unwrap()
    }
}

impl<T> Clone for MutHandle<T>{
    fn clone(&self) -> Self{
        Self{
            inner: self.inner.clone()
        }
    }
}
