use std::sync::atomic;
use std::ptr::NonNull;
use super::resource_manager::ResourceType;

#[derive(Hash)]
pub struct ResourceHandle {
    ptr: NonNull<ResourceHandleRaw>,
    resource_type: ResourceType,
}

pub struct ResourceHandleRaw{
    uuid: u64,
    rc: atomic::AtomicUsize,
}

impl ResourceHandle{
    pub fn new(resource_type: ResourceType) -> Self{
        let ptr = Box::into_raw(Box::new(ResourceHandleRaw{
            uuid: rand::random(),
            rc: atomic::AtomicUsize::new(1)
        }));

        Self{
            ptr: NonNull::new(ptr).unwrap(),
            resource_type,
        }
    }

    pub fn get_uuid(&self) -> u64{
        unsafe{
            self.ptr.as_ref().uuid
        }
    }

    pub fn get_type(&self) -> &ResourceType{
        &self.resource_type
    }
}

impl Clone for ResourceHandle{
    fn clone(&self) -> Self{
        unsafe{
            let rc = self.ptr.as_ref().rc.fetch_add(1, atomic::Ordering::Relaxed);
            if rc == 0{
                panic!("ResourceHandle is already dropped");
            }
        }

        Self{
            ptr: self.ptr,
            resource_type: self.resource_type.clone(),
        }
    }
}

impl Drop for ResourceHandle{
    fn drop(&mut self){
        unsafe{
            let rc = self.ptr.as_ref().rc.fetch_sub(1, atomic::Ordering::Release);
            if rc == 1{
                atomic::fence(atomic::Ordering::Acquire);
                let _ = Box::from_raw(self.ptr.as_ptr());
            }
        }
    }
}

impl PartialEq for ResourceHandle{
    fn eq(&self, other: &Self) -> bool{
        self.ptr == other.ptr
    }
}

impl Eq for ResourceHandle{}

impl std::fmt::Debug for ResourceHandle{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceHandle({})", self.get_uuid())
    }
}
