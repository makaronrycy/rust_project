use std::default;
//source: https://github.com/jack1232/wgpu11
use std:: {iter, mem ,collections::HashMap};
use cgmath::{ Matrix, Matrix4, Quaternion, SquareMatrix, Vector3 };
use cgmath::prelude::*;


use wgpu::util::DeviceExt;
use wgpu::BindGroupLayout;
use winit::dpi::PhysicalPosition;
use winit::keyboard;
use winit::{
    event::*,
    event_loop::EventLoop,
    window::Window,
    keyboard::{PhysicalKey,KeyCode}
};


#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use bytemuck:: {Pod, Zeroable, cast_slice};
use cgmath::prelude::*;
#[path="../src/transforms.rs"]
mod transforms;
#[path="../src/context.rs"]
mod context;

#[path="../src/resources.rs"]
mod resources;

#[path="../src/camera.rs"]
mod camera;

#[path="../src/phys.rs"]
mod phys;
use resources::model::{texture::Texture, DrawModel,DrawLight,Instance, InstanceRaw, Model, ModelVertex, Vertex,Object,Globals,Locals};
use resources::{UniformPool};
use camera::{Camera, CameraController, CameraUniform, Projection};
const ANIMATION_SPEED:f32 = 1.0;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct LightUniform {
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

struct State<'a> {
    init: context::InitWgpu<'a>,
    render_pipeline: wgpu::RenderPipeline,
    light_pipeline: wgpu::RenderPipeline,
    objects: Vec<Object>,
    global_bind_group_layout: BindGroupLayout,
    window: &'a Window,
    depth_texture: Texture,
    camera: Camera,
    obj_bind_group_layout: wgpu::BindGroupLayout,
    obj_bind_groups: HashMap<usize,wgpu::BindGroup>,
    uniform_pool: UniformPool,
    global_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    global_uniform_buffer: wgpu::Buffer,
    instance_buffers: HashMap<usize, wgpu::Buffer>,
    mouse_pressed: bool,
    physics: phys::Physics,
    projection: Projection,
}
impl <'a>State <'a>{
    
    async fn new(window: &'a Window) -> Self {        
        let init =  context::InitWgpu::init_wgpu(window).await;
        const NUMBER_OF_PINS: i32 = 10;
        let shader_module = init.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Normal Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
          let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
          let projection =
              camera::Projection::new(init.config.width, init.config.height, cgmath::Deg(45.0), 0.1, 100.0);
          let camera_controller = camera::CameraController::new(4.0, 0.4);
  
          let mut camera_uniform = CameraUniform::new();
          camera_uniform.update_view_proj(&camera, &projection);
  
          let camera_buffer = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
              label: Some("Camera Buffer"),
              contents: bytemuck::cast_slice(&[camera_uniform]),
              usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
          });
  
        // Setup global uniforms
        // Global bind group layout
        let light_size = mem::size_of::<LightUniform>() as wgpu::BufferAddress;
        let global_size = mem::size_of::<Globals>() as wgpu::BufferAddress;
        let global_bind_group_layout =
            init.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("[Phong] Globals"),
                entries: &[
                    // Global uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(global_size),
                        },
                        count: None,
                    },
                    // Lights
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(light_size),
                        },
                        count: None,
                    },
                    // Sampler for textures
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Global uniform buffer
        let global_uniform_buffer = init.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("[Phong] Globals"),
            size: global_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Create light uniforms and setup buffer for them
        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };
        let light_buffer = init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("[Phong] Lights"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        // We also need a sampler for our textures
        let sampler = init.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("[Phong] sampler"),
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        // Combine the global uniform, the lights, and the texture sampler into one bind group
        let global_bind_group = init.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("[Phong] Globals"),
            layout: &global_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: global_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Setup local uniforms
        // Local bind group layout
        let local_size = mem::size_of::<Locals>() as wgpu::BufferAddress;
        let obj_bind_group_layout =
            init.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("[Phong] Locals"),
                entries: &[
                    // Local uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(local_size),
                        },
                        count: None,
                    },
                    // Mesh texture
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        // Setup the render pipeline
        let pipeline_layout = init.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("[Phong] Pipeline"),
            bind_group_layouts: &[&global_bind_group_layout, &obj_bind_group_layout],
            push_constant_ranges: &[],
        });
        let vertex_buffers = [ModelVertex::desc(), InstanceRaw::desc()];
        let depth_stencil = Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        });



        let primitive = wgpu::PrimitiveState {
            cull_mode: Some(wgpu::Face::Back),
            
            ..Default::default()
        };
        let multisample = wgpu::MultisampleState {
            ..Default::default()
        };
        let color_format = Texture::DEPTH_FORMAT;

        let render_pipeline = init.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("[Phong] Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
                compilation_options: Default::default()
            },
            primitive,
            depth_stencil: depth_stencil.clone(),
            multisample,
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: init.config.format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache:None,
        });

        // Create depth texture
        let depth_texture =
            Texture::create_depth_texture(&init.device, &init.config, "depth_texture");


        let light_shader = init.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Light Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
        });

        let light_pipeline =
            init.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("[Phong] Light Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &light_shader,
                    entry_point: "vs_main",
                    buffers: &[ModelVertex::desc()],
                    compilation_options: Default::default(),
                },
                primitive,
                depth_stencil,
                multisample,
                cache:None,
                fragment: Some(wgpu::FragmentState {
                    module: &light_shader,
                    compilation_options: Default::default(),
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: init.config.format,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });


        //Physics
        let mut physics = phys::Physics::new();
        physics.build_colliders();

        let uniform_pool = UniformPool::new("[Phong] Locals", local_size);
        let mut objects: Vec<Object> =  Vec::new();



        //creating objects
        let ball_model =
            resources::load_model("ball.obj", &init.device, &init.queue)
                .await
                .unwrap();

        
        let pin_model =
            resources::load_model("pin.obj", &init.device, &init.queue)
                .await
                .unwrap();
            
        let mut ball_instances = Vec::new();
        ball_instances.push({Instance{position:Vector3{x: 0.0,y:0.0,z:1.0},rotation:Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(0.0)),scale:Vector3{x:5.0,y:5.0,z:5.0}}});
        let pin_instances = (0..NUMBER_OF_PINS)
        .map(|i| {
            let position = cgmath::Vector3{x: (i%4) as f32 * 0.6 - (i/4) as f32 *0.3, y:0.0, z: (i/4) as f32 * -0.8 + 10.0};

            let rotation = cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(0.0));

            let scale = cgmath::Vector3 { x: (5.0), y: (5.0), z: (5.0) };
            Instance { position, rotation,scale }
        })
        .collect::<Vec<_>>();
        objects.push(Object::new(ball_model, ball_instances,String::from("Ball")));
        objects.push(Object::new(pin_model, pin_instances,String::from("Pin")));


        println!("{}", objects.len());
        let instance_buffers = HashMap::new();
        Self {
            init,
            render_pipeline,
            light_pipeline,
            global_bind_group,
            objects,
            window,
            depth_texture,
            camera,
            obj_bind_group_layout,
            obj_bind_groups: Default::default(),
            uniform_pool,
            global_bind_group_layout,
            camera_controller,
            camera_uniform,
            global_uniform_buffer,
            instance_buffers,
            mouse_pressed: false,
            physics,
            projection
        }
    }
    pub fn window(&self) -> &Window {
        &self.window
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.projection.resize(new_size.width, new_size.height);
            self.init.config.width = new_size.width;
            self.init.config.height = new_size.height;
            self.init.size = new_size;
            self.init.surface.configure(&self.init.device, &self.init.config);
            self.depth_texture =
                Texture::create_depth_texture(&self.init.device, &self.init.config, "depth_texture");
        }

    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }
    fn fixed_update(&mut self){
        self.physics.simulate();
    }
    fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera,dt);
        self.camera_uniform.update_view_proj(&self.camera,&self.projection);
        self.init.queue.write_buffer(
            &self.global_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        // update uniform buffer
        let dt = ANIMATION_SPEED * dt.as_secs_f32();
        //transforms on model
        // Update local uniforms
        let mut obj_index = 0;
        for obj in &mut self.objects {
            obj.locals.position = self.physics.get_translation(obj_index);
            self
                .uniform_pool
                .update_uniform(obj_index, obj.locals, &self.init.queue);
                
            obj_index += 1;
        }
        
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        //let output = self.init.surface.get_current_frame()?.output;
        let output = self.init.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        
        let depth_texture = self.init.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.init.config.width,
                height: self.init.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format:wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .init.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.2,
                            g: 0.247,
                            b: 0.314,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                //depth_stencil_attachment: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            if (self.uniform_pool.buffers.len() < self.objects.len()) {
                self.uniform_pool.alloc_buffers(self.objects.len(), &self.init.device);
            }
            let mut obj_index = 0;
            for obj in &self.objects{
                let local_buffer =&self.uniform_pool.buffers[obj_index];
                self.obj_bind_groups.entry(obj_index).or_insert_with(||{
                    self.init.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Instance bind group"),
                        layout: &self.obj_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: local_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(
                                    &obj.model.materials[0].diffuse_texture.view,
                                ),
                            },
                        ],
                    })
                });
                self.instance_buffers.entry(obj_index).or_insert_with(|| {
                    // We condense the matrix properties into a flat array (aka "raw data")
                    // (which is how buffers work - so we can "stride" over chunks)
                    let instance_data = obj
                        .instances
                        .iter()
                        .map(Instance::to_raw)
                        .collect::<Vec<_>>();
                    // Create the instance buffer with our data
                    let instance_buffer =
                        self.init.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Instance Buffer"),
                            contents: bytemuck::cast_slice(&instance_data),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                    instance_buffer
                });
                obj_index+=1;
            }
               
                
            obj_index = 0;
            render_pass.set_pipeline(&self.light_pipeline);
            
            render_pass.draw_light_model(&self.objects[0].model, &self.global_bind_group,&self.obj_bind_groups.get(&0).expect("No obj bind group found for lighting"));
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.global_bind_group, &[]);

            
            for obj in &self.objects{
                render_pass.set_vertex_buffer(1, self.instance_buffers[&obj_index].slice(..));
                render_pass.draw_model_instanced(&obj.model, 0.. obj.instances.len() as u32, &self.obj_bind_groups[&obj_index]);
                obj_index+=1;
            }
        }
            

        self.init.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub fn run(title: &str) {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = winit::window::WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title(title);

    let mut state = pollster::block_on(State::new(&window));    
    let mut last_render_time: std::time::Instant = std::time::Instant::now();
    let mut last_physics_sim: std::time::Instant = std::time::Instant::now();
    let mut dtphysics = std::time::Instant::now() - last_physics_sim;

    event_loop.run(move |event, control_flow: &winit::event_loop::EventLoopWindowTarget<()>| {
        match event {
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => if state.mouse_pressed {
                state.camera_controller.process_mouse(delta.0, delta.1)
            }
            
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        
                        WindowEvent::CloseRequested
                        //Keyboard events
                        | WindowEvent::KeyboardInput {
                            event:
                                
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                                
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            let now = std::time::Instant::now();
                            let dt = now - last_render_time;
                            if(dtphysics.as_secs_f32()> 0.02){
                                state.fixed_update();
                                last_physics_sim = now;
                            } else{
                                dtphysics = now - last_physics_sim;
                            }
                            last_render_time = now;
                            state.update(dt);
                            
                            
                            state.window().request_redraw();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.init.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                    
                }
            }
            _ => {}
        }
    }).unwrap();
}