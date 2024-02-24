use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, cmp::max, mem::size_of, sync::Arc};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBinding,
    BufferBindingType, BufferSize, BufferUsages, Color, CommandEncoderDescriptor, Device,
    DeviceDescriptor, FragmentState, IndexFormat, Instance, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration,
    TextureViewDescriptor, VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::window::Window;

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
pub struct Renderer {
    device: Device,
    surface: Surface<'static>,
    queue: Queue,
    render_pipeline: RenderPipeline,
    config: SurfaceConfiguration,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    resolution_buffer: Buffer,
    time_buffer: Buffer,
    bind_group_global_input: BindGroup,
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
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None)
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
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(2 * size_of::<u32>() as u64),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1, // time
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(size_of::<u32>() as u64),
                        },
                        count: None,
                    },
                ],
            });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout_global_input],
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
        let resolution_buffer_size = 2 * size_of::<u32>() as BufferAddress;
        let resolution_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[size.width, size.height]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let time_buffer_size = size_of::<u32>() as BufferAddress;
        let time_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&[0u32]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind_group_global_input = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_global_input,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &resolution_buffer,
                        offset: 0,
                        size: BufferSize::new(resolution_buffer_size),
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &time_buffer,
                        offset: 0,
                        size: BufferSize::new(time_buffer_size),
                    }),
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
            time_buffer,
            bind_group_global_input,
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
    }

    pub fn set_time(&mut self, time: u32) {
        self.queue
            .write_buffer(&self.time_buffer, 0, bytemuck::bytes_of(&[time]));
    }

    pub fn draw(&mut self) {
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
        rpass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.draw_indexed(0..6, 0, 0..1);
        drop(rpass);
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
