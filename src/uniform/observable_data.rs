use std::sync::{Arc, Mutex};

/// # Observable Data
/// 
/// Represents data that can be observed for changes
/// 
/// * `T` - The type of the data
/// 
/// # Fields
/// 
/// * `data` - The data
/// 
/// * `dirty` - Whether the data has been changed
pub struct ObservableData<T> {
    data: T,
    dirty: Arc<Mutex<bool>>,
}

impl<T> ObservableData<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            dirty: Arc::new(Mutex::new(false)),
        }
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn set(&mut self, value: T) {
        self.data = value;
        let mut dirty = self.dirty.lock().unwrap();
        *dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        let dirty = self.dirty.lock().unwrap();
        *dirty
    }

    pub fn clear_dirty(&mut self) {
        let mut dirty = self.dirty.lock().unwrap();
        *dirty = false;
    }
}
