/*use wgpu::*;

pub struct OwnedTexture {
    pub texture: Texture,
    pub format: TextureFormat,
    pub size: Extent3d,
}


pub struct StorageTexture<'a> {
    pub view: TextureView,
    pub access: StorageTextureAccess,
    pub texture: &'a OwnedTexture
}

pub trait WgpuBindable {
    fn binding_type(&self) -> BindingType;
    fn bind(&self) -> BindingResource;
}

impl<'a> WgpuBindable for StorageTexture<'a> {
    fn binding_type(&self) -> BindingType {
        BindingType::StorageTexture {
            access: self.access,
            format: self.texture.format,
            view_dimension: TextureViewDimension::D2,
        }
    }

    fn bind(&self) -> BindingResource {
        BindingResource::TextureView(&self.view)
    }
}

pub struct BindGroupBuilder<'a> {
    layout: Vec<BindGroupLayoutEntry>,
    binds: Vec<BindGroupEntry<'a>>,
}

fn texture_sample_type(format: TextureFormat) -> TextureSampleType {
    format.describe().sample_type
}

impl Default for BindGroupBuilder<'_> {
    fn default() -> Self {
        BindGroupBuilder {
            layout: vec![],
            binds: vec![],
        }
    }
}
impl<'a> BindGroupBuilder<'a> {
    pub fn bind<T: WgpuBindable>(mut self, value: &'a T) -> Self {
        let binding = self.layout.len() as u32;

        self.layout.push(BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::COMPUTE,
            ty: value.binding_type(),
            count: None
        });
        self.binds.push(BindGroupEntry {
            binding,
            resource: value.bind(),
        });
        self
    }

    pub fn build(&self, device: &mut wgpu::Device) -> BindGroup {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &self.layout,
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &self.binds,
        })
    }
}
*/