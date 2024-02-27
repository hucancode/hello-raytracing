use bytemuck::{bytes_of, Pod, Zeroable};
use std::{borrow::Cow, cmp::{max, min}, mem::size_of, sync::Arc};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor,
    BufferUsages, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, FragmentState,
    IndexFormat, Instance, Limits, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StoreOp, Surface, SurfaceConfiguration, TextureViewDescriptor,
    VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::window::Window;

use crate::scene::{Camera, Scene, Sphere};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 4],
}
impl Vertex {
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &vertex_attr_array![0 => Float32x4],
        }
    }
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [1.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0, 1.0],
    },
];
const INDICES: &[u32] = &[0, 1, 2, 2, 3, 0];
const MAX_IMAGE_BUFFER_SIZE: u32 = 4096 * 4096;
const MAX_OBJECT_IN_SCENE: u64 = 100;

pub struct Renderer {
    device: Device,
    surface: Surface<'static>,
    queue: Queue,
    render_pipeline: RenderPipeline,
    config: SurfaceConfiguration,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    resolution_buffer: Buffer,
    frame_count_buffer: Buffer,
    time_buffer: Buffer,
    scene_object_buffer: Buffer,
    camera_buffer: Buffer,
    bind_group_global_input: BindGroup,
    bind_group_scene: BindGroup,
    frame_count: u32,
}
impl Renderer {
    pub async fn new(window: Arc<Window>) -> Renderer {
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);
        let instance = Instance::default();
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");
        let mut limits = Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
        let max_storage_buffer_size = 256 << 20;
        limits.max_buffer_size = max(limits.max_buffer_size, max_storage_buffer_size as u64);
        limits.max_storage_buffer_binding_size = max_storage_buffer_size;
        limits.max_storage_buffers_per_shader_stage = 4;
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_limits: limits,
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("Failed to create device");
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&INDICES),
            usage: BufferUsages::INDEX,
        });
        let bind_group_layout_global_input =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0, // resolution
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1, // frame count
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2, // time
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3, // image data
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let bind_group_layout_scene = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0, // scene objects
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1, // camera
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        println!("creating pipeline layout");
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout_global_input, &bind_group_layout_scene],
            ..Default::default()
        });
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });
        println!("created pipeline");
        let resolution_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[size.width, size.height]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let frame_count_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&[0u32]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let time_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&[0u32]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let image_data = vec![0f32; MAX_IMAGE_BUFFER_SIZE as usize];
        let image_buffer = device.create_buffer_init(&BufferInitDescriptor {
            contents: bytemuck::cast_slice(image_data.as_slice()),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            label: None,
        });
        let bind_group_global_input = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_global_input,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: resolution_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: frame_count_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: time_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: image_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });
        let scene_object_buffer = device.create_buffer(&BufferDescriptor {
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            size: MAX_OBJECT_IN_SCENE * size_of::<Sphere>() as BufferAddress,
            mapped_at_creation: false,
            label: None,
        });
        let camera_buffer = device.create_buffer(&BufferDescriptor {
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size: size_of::<Camera>() as BufferAddress,
            mapped_at_creation: false,
            label: None,
        });
        let bind_group_scene = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_scene,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: scene_object_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        Self {
            device,
            surface,
            queue,
            render_pipeline,
            config,
            vertex_buffer,
            index_buffer,
            resolution_buffer,
            frame_count_buffer,
            time_buffer,
            scene_object_buffer,
            camera_buffer,
            bind_group_global_input,
            bind_group_scene,
            frame_count: 0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = max(1, width);
        self.config.height = max(1, height);
        self.surface.configure(&self.device, &self.config);
        self.queue.write_buffer(
            &self.resolution_buffer,
            0,
            bytemuck::bytes_of(&[width, height]),
        );
        self.frame_count = 0;
    }

    pub fn set_scene(&mut self, scene: &Scene) {
        self.queue
            .write_buffer(&self.camera_buffer, 0, bytes_of(&scene.camera));
        let data = bytemuck::cast_slice(&scene.objects.as_slice());
        let n = min(data.len(), MAX_OBJECT_IN_SCENE as usize * size_of::<Sphere>());
        self.queue
            .write_buffer(&self.scene_object_buffer, 0, &data[0..n]);
    }

    pub fn set_time(&mut self, time: u32) {
        self.queue
            .write_buffer(&self.time_buffer, 0, bytemuck::bytes_of(&[time]));
    }

    pub fn draw(&mut self) {
        self.queue.write_buffer(
            &self.frame_count_buffer,
            0,
            bytemuck::bytes_of(&[self.frame_count]),
        );
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });
        let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group_global_input, &[]);
        rpass.set_bind_group(1, &self.bind_group_scene, &[]);
        rpass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.draw_indexed(0..6, 0, 0..1);
        drop(rpass);
        self.queue.submit(Some(encoder.finish()));
        frame.present();
        self.frame_count += 1;
    }
}
