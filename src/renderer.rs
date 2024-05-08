use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, cmp::max, mem::size_of, sync::Arc};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferAddress, BufferBindingType, BufferDescriptor,
    BufferUsages, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, FragmentState,
    IndexFormat, Instance, Limits, LoadOp, MultisampleState, Operations, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StoreOp, Surface, SurfaceConfiguration, TextureViewDescriptor,
    VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::window::Window;

use crate::scene::Camera;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
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
const MAX_IMAGE_BUFFER_SIZE: usize = 4096 * 4096;

pub struct Buffers {
    buffers: Vec<wgpu::Buffer>,
    group: wgpu::BindGroup,
}

pub struct Renderer {
    device: Device,
    surface: Surface<'static>,
    queue: Queue,
    render_pipeline: RenderPipeline,
    config: SurfaceConfiguration,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    buffers: Vec<Buffers>,
    frame_count: u32,
}
impl Renderer {
    pub async fn new(
        window: Arc<Window>,
        custom_buffers: Vec<(BufferBindingType, u64)>,
        shader_source: &str,
    ) -> Self {
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
            source: ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
        });
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });
        let builtin_buffer = vec![
            (BufferBindingType::Uniform, 2 * size_of::<u32>() as u64), // resolution
            (BufferBindingType::Uniform, size_of::<u32>() as u64),     // frame count
            (BufferBindingType::Uniform, size_of::<u32>() as u64),     // time
            (
                BufferBindingType::Storage { read_only: false },
                (MAX_IMAGE_BUFFER_SIZE * size_of::<u32>()) as u64,
            ), // image data
            (BufferBindingType::Uniform, size_of::<Camera>() as u64),  // camera
        ];
        let buffers = [builtin_buffer, custom_buffers];
        let bind_group_layouts: Vec<wgpu::BindGroupLayout> = buffers
            .iter()
            .map(|group| {
                let entries: Vec<_> = group
                    .iter()
                    .enumerate()
                    .map(|(binding, &(ty, _))| BindGroupLayoutEntry {
                        binding: binding as u32,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    })
                    .collect();
                device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: entries.as_slice(),
                })
            })
            .collect();
        let bind_group_layouts: Vec<_> = bind_group_layouts.iter().collect();
        println!("creating pipeline layout");
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &bind_group_layouts,
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
        let buffers: Vec<_> = buffers
            .iter()
            .enumerate()
            .map(|(i, group)| {
                let buffers: Vec<_> = group
                    .iter()
                    .map(|&(ty, size)| {
                        let usage = match ty {
                            BufferBindingType::Storage { read_only: _ } => {
                                BufferUsages::STORAGE | BufferUsages::COPY_DST
                            }
                            BufferBindingType::Uniform => {
                                BufferUsages::UNIFORM | BufferUsages::COPY_DST
                            }
                        };
                        device.create_buffer(&BufferDescriptor {
                            usage,
                            size,
                            mapped_at_creation: false,
                            label: None,
                        })
                    })
                    .collect();
                let entries: Vec<_> = group
                    .iter()
                    .enumerate()
                    .map(|(binding, _)| BindGroupEntry {
                        binding: binding as u32,
                        resource: buffers[binding].as_entire_binding(),
                    })
                    .collect();
                let group = device.create_bind_group(&BindGroupDescriptor {
                    layout: bind_group_layouts[i],
                    entries: entries.as_slice(),
                    label: None,
                });
                Buffers { buffers, group }
            })
            .collect();
        println!("created pipeline");

        let buffer = &buffers[0].buffers[0];
        queue.write_buffer(buffer, 0, bytemuck::bytes_of(&[size.width, size.height]));

        Self {
            device,
            surface,
            queue,
            render_pipeline,
            config,
            vertex_buffer,
            index_buffer,
            buffers,
            frame_count: 0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = max(1, width);
        self.config.height = max(1, height);
        self.surface.configure(&self.device, &self.config);
        let buffer = &self.buffers[0].buffers[0];
        self.queue
            .write_buffer(buffer, 0, bytemuck::bytes_of(&[width, height]));
        self.frame_count = 0;
    }

    pub fn set_time(&mut self, time: u32) {
        let buffer = &self.buffers[0].buffers[2];
        self.queue
            .write_buffer(buffer, 0, bytemuck::bytes_of(&[time]));
    }
    pub fn set_frame_count(&mut self, n: u32) {
        let buffer = &self.buffers[0].buffers[1];
        self.queue.write_buffer(buffer, 0, bytemuck::bytes_of(&[n]));
    }
    pub fn set_camera(&mut self, camera: &Camera) {
        let buffer = &self.buffers[0].buffers[4];
        self.queue
            .write_buffer(buffer, 0, bytemuck::bytes_of(camera))
    }

    pub fn write_buffer(&mut self, data: &[u8], buffer: usize) {
        let buffer = &self.buffers[1].buffers[buffer];
        self.queue.write_buffer(buffer, 0, data)
    }

    pub fn draw(&mut self) {
        self.set_frame_count(self.frame_count);
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
        for (i, group) in self.buffers.iter().enumerate() {
            rpass.set_bind_group(i as u32, &group.group, &[]);
        }
        rpass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.draw_indexed(0..6, 0, 0..1);
        drop(rpass);
        self.queue.submit(Some(encoder.finish()));
        frame.present();
        self.frame_count += 1;
    }
}
