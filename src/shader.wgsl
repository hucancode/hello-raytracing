const PI = 3.141592653589793;
const PI2 = PI*2;
const start_color = vec4(0.4, 0.7, 1.0, 1.0);
const end_color = vec4(0.0, 0.1, 0.3, 1.0);

@group(0) @binding(0)
var<uniform> resolution: vec2<u32>;
@group(0) @binding(1)
var<uniform> time: u32;
@vertex
fn vs_main(@location(0) position: vec4<f32>) -> @builtin(position) vec4<f32> {
  return position;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
  let k = position.y/f32(resolution.y)*0.5+0.25;
  let offset = sin(f32(time)*0.001)*0.25;
  return mix(start_color, end_color, k+offset);
}
