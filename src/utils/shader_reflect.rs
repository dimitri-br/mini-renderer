use regex::Regex;
use std::collections::HashMap;
use log::{error, info};

#[derive(Debug, Clone)]
pub enum BindingType{
    Texture,
    TextureSampler,
    Uniform,
    Storage
}

#[derive(Debug, Clone)]
pub struct Binding{
    group: u32,
    binding: u32,
    name: String,
    binding_type: BindingType
}



pub struct ShaderReflect{
    source: String,
    bindings: HashMap<String, Binding>
}

impl ShaderReflect{
    pub fn new<T: Into<String>>(source: T) -> Self{
        Self{
            source: source.into(),
            bindings: HashMap::new()
        }
    }

    pub fn reflect(&mut self) {
        let re_tex = Regex::new(r"@group\(\s*(\d+)\s*\)\s*@binding\(\s*(\d+)\s*\)\s*var\s+(\w+)\s*:\s*([^<;]+)").unwrap();
        for capture in re_tex.captures_iter(&self.source){
            let group = capture[1].parse::<u32>().unwrap();
            let binding = capture[2].parse::<u32>().unwrap();
            let name = &capture[3];
            let tex_type = &capture[4];

            if tex_type.contains("sampler") {
                self.bindings.insert(name.to_string(), Binding {
                    group,
                    binding,
                    name: name.to_string(),
                    binding_type: BindingType::TextureSampler
                });
            } else {
                self.bindings.insert(name.to_string(), Binding {
                    group,
                    binding,
                    name: name.to_string(),
                    binding_type: BindingType::Texture
                });
            }
        }

        // get wgsl uniform bindings
        let re_binding_type = Regex::new(r"@group\(\s*(\d+)\s*\)\s*@binding\(\s*(\d+)\s*\)\s*var\s*\W(\w+)\W\s*(\w+)\s*:\s*(\w*)").unwrap();
        for capture in re_binding_type.captures_iter(&self.source){
            let group = capture[1].parse::<u32>().unwrap();
            let binding = capture[2].parse::<u32>().unwrap();
            let bind_type = &capture[3];
            let name = &capture[4];

            let binding_type = match bind_type{
                "uniform" => BindingType::Uniform,
                "storage" => BindingType::Storage,
                _ => {
                    error!("Unknown binding type: {}", bind_type);
                    panic!("Unknown binding type");
                }
            };

            self.bindings.insert(name.to_string(), Binding {
                group,
                binding,
                name: name.to_string(),
                binding_type
            });
        }

        println!("{:?}", self.bindings);
    }

    pub fn get_bindings(&self) -> HashMap<String, Binding>{
        self.bindings.clone()
    }
}
