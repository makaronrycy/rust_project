use std::io::{BufReader, Cursor};

use wgpu::util::DeviceExt;


pub mod model;
use model::{Instance};
use model::texture::Texture;
pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let txt = std::fs::read_to_string(path)?;
    Ok(txt)
    }
    


pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let path = std::path::Path::new(env!("OUT_DIR"))
        .join("res")
        .join(file_name);
    let data = std::fs::read(path)?;
    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<Texture> {
    let data = load_binary(file_name).await?;
    Texture::from_bytes(device, queue, &data, file_name)
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader: BufReader<Cursor<String>> = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text: String = load_string(&p).await.unwrap();
            println!("{}",mat_text);
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let diffuse_texture = load_texture(&m.diffuse_texture, device, queue).await?;
        materials.push(model::Material {
            name: m.name,
            diffuse_texture,
            
        })
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
            .map(|i| {
                if m.mesh.normals.is_empty(){
                    model::ModelVertex {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                        normal: [0.0, 0.0, 0.0],
                    }
                }else{
                    model::ModelVertex {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                        normal: [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ],
                    }
                }
            })
            .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            log::info!("Mesh: {}", m.name);
            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}

pub struct UniformPool {
    label: &'static str,
    pub buffers: Vec<wgpu::Buffer>,
    size: u64,
}

impl UniformPool {
    pub fn new(label: &'static str, size: u64) -> Self {
        Self {
            label,
            buffers: Vec::new(),
            size,
        }
    }

    pub fn alloc_buffers(&mut self, count: usize, device: &wgpu::Device) {
        // We reset the buffers each time we allocate
        // TODO: Ideally we should keep track of the object it belongs to,
        // so we can add/remove objects (and their uniform buffers) dynamically
        self.buffers = Vec::new();

        for _ in 0..count {
            let local_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&self.label),
                size: self.size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.buffers.push(local_uniform_buffer);
        }
    }

    pub fn update_uniform<T: bytemuck::Pod>(&self, index: usize, data: T, queue: &wgpu::Queue) {
        if &self.buffers.len() > &0 {
            queue.write_buffer(&self.buffers[index], 0, bytemuck::cast_slice(&[data]));
        }
    }
}