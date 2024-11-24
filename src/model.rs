#[path ="../src/texture.rs"]
pub mod texture;
use std::default;
use std::ops::Range;

use cgmath::Matrix4;
use cgmath::Rad;
use cgmath::Vector3;
use cgmath::prelude::*;
use wgpu::BindGroup;
pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
    ambient: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Locals {
    pub model_mat: [f32;16],
    pub color: [f32; 4],
    pub normal: [f32; 4],
    pub lights: [f32; 4],

}
impl Locals{
    pub fn create_transforms(&mut self,translation:[f32; 3], rotation:[f32; 3], scaling:[f32; 3]){

        // create transformation matrices
        let trans_mat = Matrix4::from_translation(Vector3::new(translation[0], translation[1], translation[2]));
        let rotate_mat_x = Matrix4::from_angle_x(Rad(rotation[0]));
        let rotate_mat_y = Matrix4::from_angle_y(Rad(rotation[1]));
        let rotate_mat_z = Matrix4::from_angle_z(Rad(rotation[2]));
        let scale_mat = Matrix4::from_nonuniform_scale(scaling[0], scaling[1], scaling[2]);
    
        let m = (trans_mat * rotate_mat_z * rotate_mat_y * rotate_mat_x * scale_mat);
        //unfortunately have do to this conversion to send pod to gpu
        self.model_mat= *m.as_ref();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
}

impl Vertex for InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4 for Vertex. We'll start at slot 5 to not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code, though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // NEW!
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        let model =
            cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)*cgmath::Matrix4::from_nonuniform_scale(self.scale.x,self.scale.y,self.scale.z);
        InstanceRaw {
            model: model.into(),
            // NEW!
            normal: cgmath::Matrix3::from(self.rotation).into(),
        }
    }
    pub fn translate(&mut self, translation: Vector3<f32>){
        self.position += translation;
    }
    pub fn rotate(&mut self, axis: Vector3<f32>, angle_rad: f32) {
        let rotation = cgmath::Quaternion::from_axis_angle(axis.normalize(), cgmath::Rad(angle_rad));
        self.rotation = rotation * self.rotation;
    }
    pub fn set_scale(&mut self, scale: cgmath::Vector3<f32>) {
        self.scale = scale;
    }
}
pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    // pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        local_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        local_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_model(&mut self, model: &'a Model, local_bind_group: &'a wgpu::BindGroup);
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        local_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        local_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, local_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        local_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(1, local_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(&mut self, model: &'b Model, local_bind_group: &'b wgpu::BindGroup) {
        self.draw_model_instanced(model, 0..1, local_bind_group);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        local_bind_group: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone(), local_bind_group);
        }
    }
}
pub trait DrawLight<'a> {
    fn draw_light_mesh(
        &mut self,
        mesh: &'a Mesh,
        global_bind_group: &'a wgpu::BindGroup,
        local_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        global_bind_group: &'a wgpu::BindGroup,
        local_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_light_model(
        &mut self,
        model: &'a Model,
        global_bind_group: &'a wgpu::BindGroup,
        local_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        global_bind_group: &'a wgpu::BindGroup,
        local_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawLight<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_light_mesh(
        &mut self,
        mesh: &'b Mesh,
        global_bind_group: &'b wgpu::BindGroup,
        local_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_light_mesh_instanced(mesh, 0..1, global_bind_group, local_bind_group);
    }

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        global_bind_group: &'b wgpu::BindGroup,
        local_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, global_bind_group, &[]);
        self.set_bind_group(1, local_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_light_model(
        &mut self,
        model: &'b Model,
        global_bind_group: &'b wgpu::BindGroup,
        local_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_light_model_instanced(model, 0..1, global_bind_group, local_bind_group);
    }
    fn draw_light_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        global_bind_group: &'b wgpu::BindGroup,
        local_bind_group: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_light_mesh_instanced(
                mesh,
                instances.clone(),
                global_bind_group,
                local_bind_group,
            );
        }
    }
}
pub struct Object{
    pub id: String,
    pub model: Model,
    pub instances: Vec<Instance>,
    pub locals: Locals,
}
impl Object{
    pub fn new(model: Model, instances: Vec<Instance>,name: String) ->Self{
        let trans_mat = Matrix4::from_translation(Vector3::new(0.0, 0.0,0.0));
        let rotate_mat_x = Matrix4::from_angle_x(Rad(0.0));
        let rotate_mat_y = Matrix4::from_angle_y(Rad(0.0));
        let rotate_mat_z = Matrix4::from_angle_z(Rad(0.0));
        let scale_mat = Matrix4::from_nonuniform_scale(1.0, 1.0, 1.0);
        let m = (trans_mat * rotate_mat_z * rotate_mat_y * rotate_mat_x * scale_mat);
        //unfortunately have do to this conversion to send pod to gpu
        let model_mat: [f32;16] = *m.as_ref();
        Self{model:(model),instances:(instances),id:(name), locals:(Locals { model_mat,color: ([0.0, 0.0, 1.0, 1.0]),normal: ([0.0, 0.0, 0.0, 0.0]),lights: ([0.0, 0.0, 0.0, 0.0]),})}
    }
}