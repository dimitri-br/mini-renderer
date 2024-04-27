use crate::AsBytes;

#[derive(Debug, Clone)]
pub struct Transform{
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3
}

impl Transform{
    pub fn new() -> Self{
        Self{
            position: glam::Vec3::new(0.0, 0.0, 0.0),
            rotation: glam::Quat::from_euler(glam::EulerRot::YXZ, 0.0, 0.0, 0.0),
            scale: glam::Vec3::new(1.0, 1.0, 1.0)
        }
    }

    pub fn get_matrix(&self) -> glam::Mat4{
        glam::Mat4::from_translation(self.position) * glam::Mat4::from_quat(self.rotation) * glam::Mat4::from_scale(self.scale)
    }

    pub fn get_position(&self) -> glam::Vec3 {
        self.position
    }

    pub fn get_rotation(&self) -> glam::Quat {
        self.rotation
    }

    pub fn get_scale(&self) -> glam::Vec3 {
        self.scale
    }

    pub fn set_position(&mut self, position: glam::Vec3) {
        self.position = position;
    }

    pub fn set_rotation(&mut self, rotation: glam::Quat) {
        self.rotation = rotation;
    }

    pub fn set_scale(&mut self, scale: glam::Vec3) {
        self.scale = scale;
    }
}

impl Into<TransformUniform> for Transform{
    fn into(self) -> TransformUniform {
        TransformUniform::new(&self)
    }
}

pub struct TransformUniform{
    pub transform: [[f32; 4]; 4]
}

impl TransformUniform{
    pub fn new(transform: &Transform) -> Self{
        Self{
            transform: transform.get_matrix().to_cols_array_2d()
        }
    }
}

impl AsBytes for TransformUniform{
    fn as_bytes(&self) -> &[u8] {
        unsafe{
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                std::mem::size_of::<Self>()
            )

        }
    }
}