const PI = 3.141592653589793;
const PI2 = PI*2;
const EPSILON = 0.000001;
const FLT_MAX = 3.40282e+38;
const MAT_LAMBERTIAN = 1u;
const MAT_METAL = 2u;
const MAT_DIELECTRIC = 3u;
const SKY = vec3f(0.54, 0.86, 0.92);
const BLUE = vec3f(0.54, 0.7, 0.98);
const SAMPLE_FRAME = 1000;
const SAMPLE_PER_FRAME = 1;
const BOUNCE_MAX = 10;

@group(0) @binding(0)
var<uniform> resolution: vec2u;
@group(0) @binding(1)
var<uniform> frame_count: u32;
@group(0) @binding(2)
var<uniform> time: u32;
@group(0) @binding(3)
var<storage, read_write> image: array<f32>;
@group(1) @binding(0) 
var<storage> scene: array<Sphere>;
@group(1) @binding(1) 
var<uniform> camera: Camera;

struct Camera {
  eye: vec4f,
  direction: vec4f,
  up: vec4f,
  right: vec4f,
  focal_length: f32,
  focal_blur_amount: f32,
  fov: f32,
  // _padding: vec3f,
}
struct Ray {
  origin: vec3f,
  direction: vec3f,
}
struct Light {
  center: vec3f,
  color: vec3f,
  strength: f32,
}
struct Sphere {
  center_and_radius: vec4f,
  material: Material,
  // _padding: vec4f,
}
struct HitRecord {
  point: vec3f,
  normal: vec3f,
  t: f32,
  material: Material,
  front_face: bool,
}
struct Material {
    albedo: vec4f,
    id: u32,
    params: vec3f,
    // _padding: f32,
}

@vertex
fn vs_main(@location(0) position: vec4f) -> @builtin(position) vec4f {
  return position;
}

fn rng_int(state: ptr<function, u32>) {
    // PCG random number generator
    // Based on https://www.shadertoy.com/view/XlGcRh
    let oldState = *state + 747796405u + 2891336453u;
    let word = ((oldState >> ((oldState >> 28u) + 4u)) ^ oldState) * 277803737u;
    *state = (word >> 22u) ^ word;
}
fn rng_float(state: ptr<function, u32>) -> f32 {
    rng_int(state);
    return f32(*state) / f32(0xffffffffu);
}
fn rng_vec2(state: ptr<function, u32>) -> vec2f {
    return vec2f(rng_float(state), rng_float(state));
}
fn rng_vec3(state: ptr<function, u32>) -> vec3f {
    return vec3f(rng_float(state), rng_float(state), rng_float(state));
}
fn point_on_ray(ray: Ray, t: f32) -> vec3f {
  return ray.origin + t * ray.direction;
}
fn random_on_hemisphere(state: ptr<function, u32>, normal: vec3f) -> vec3f {
  let t = f32(time);
  let v = normalize(rng_vec3(state));
  if length(v) < EPSILON {
    return normal;
  }
  if(dot(v, normal) > 0) {
    return v;
  }
  return -v;
}
fn random_on_disk(state: ptr<function, u32>, radius: f32) -> vec3f {
  let v = normalize(rng_vec2(state));
  let r = rng_float(state)*radius;
  return vec3f(v, 0.0)*r;
}
fn make_ray(uv: vec2f, state: ptr<function, u32>) -> Ray {
    let k = tan(camera.fov*0.5);
    let x = camera.right*uv.x*k;
    let y = camera.up*uv.y*k;
    let z = camera.direction;
    var direction = normalize(x+y+z);
    var origin = camera.eye;
    // focus blur
    let focus_point = origin + direction*camera.focal_length;
    origin += vec4f(random_on_disk(state, camera.focal_blur_amount), 1.0);
    direction = focus_point - origin;
    return Ray(origin.xyz, direction.xyz);
}
fn intersect_sphere(ray: Ray, sphere: Sphere) -> HitRecord {
  let center = sphere.center_and_radius.xyz;
  let radius = sphere.center_and_radius.w;
  let oc = ray.origin - center;
  let a = dot(ray.direction, ray.direction);
  let b = 2 * dot(oc, ray.direction);
  let c = dot(oc, oc) - radius*radius;
  let discriminant = b*b - 4*a*c;
  if (discriminant < 0) {
      return HitRecord(vec3f(), vec3f(), -1, sphere.material, false);
  }
  let t = (-b - sqrt(discriminant) ) / (2*a);
  let hit_point = point_on_ray(ray, t);
  var normal = (hit_point - center) / radius;
  let front_face = dot(ray.direction, normal) < 0;
  if !front_face {
    normal = -normal;
  }
  return HitRecord(hit_point, normal, t, sphere.material, front_face);
}
fn refract(uv: vec3f, n: vec3f, ir: f32) -> vec3f {
    let cos_theta = min(dot(-uv, n), 1.0);
    let r_out_perp =  ir * (uv + cos_theta*n);
    let len = length(r_out_perp);
    let r_out_parallel = -sqrt(abs(1.0 - len*len)) * n;
    return r_out_perp + r_out_parallel;
}
fn scatter(state: ptr<function, u32>, ray: Ray, hit: HitRecord) -> Ray {
  switch hit.material.id {
    case MAT_LAMBERTIAN: {
      let direction = random_on_hemisphere(state, hit.normal);
      return Ray(hit.point, direction);
    }
    case MAT_METAL: {
      let fuzziness = hit.material.params.x;
      let direction = reflect(normalize(ray.direction), hit.normal) + fuzziness * random_on_hemisphere(state, hit.normal);
      return Ray(hit.point, normalize(direction));
    }
    case MAT_DIELECTRIC: {
      var ir = hit.material.params.x;
      if hit.front_face && ir > EPSILON {
        ir = 1.0/ir;
      }
      let direction = refract(ray.direction, hit.normal, ir);
      return Ray(hit.point, normalize(direction));
    }
    default: {
      return Ray(hit.point, vec3f(1.0, 0.0, 0.0));
    }
  }
}
fn trace(ray: Ray, state: ptr<function, u32>) -> vec3f {
  var ret = vec4f(0);
  var attenuation = vec3f(1);
  var current_ray = ray;
  var first_hit = true;
  for(var b = 0;b < BOUNCE_MAX; b++) {
    var closest_hit: HitRecord;
    closest_hit.t = FLT_MAX;
    for (var i = 0u; i < arrayLength(&scene); i++) {
      let obj = scene[i];
      let hit = intersect_sphere(current_ray, obj);
      if hit.t > 0 && hit.t < closest_hit.t {
        closest_hit = hit;
      }
    }
    if abs(closest_hit.t - FLT_MAX) < EPSILON {
      break;
    }
    current_ray = scatter(state, current_ray, closest_hit);
    if first_hit {
      attenuation = closest_hit.material.albedo.rgb;
      first_hit = false;
    } else {
      attenuation *= closest_hit.material.albedo.rgb;
    }
  }
  let sky = mix(SKY, BLUE, ray.direction.y*0.5 + 0.5);
  return attenuation * sky;
}


@fragment
fn fs_main_test_rng(@builtin(position) position: vec4f) -> @location(0) vec4f {
  var rng_state = (u32(position.x)*resolution.y + u32(position.y)) * time;
  return vec4f(rng_vec3(&rng_state), 1.0);
}
@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
  var rng_state = (u32(position.x)*resolution.y + u32(position.y)) * time;
  let aspect_ratio = f32(resolution.x) / f32(resolution.y);
  let position_aa = position.xy +normalize(rng_vec2(&rng_state));
  var uv = position_aa / (vec2f(resolution) - vec2f(1));
  uv = (2 * uv - vec2(1)) * vec2(aspect_ratio, -1);
  let ray = make_ray(uv, &rng_state);
  var color = vec3f(0);
  for (var i = 0; i < SAMPLE_PER_FRAME; i += 1) {
    color += trace(ray, &rng_state);
  }
  color /= f32(SAMPLE_PER_FRAME);
  let x = u32(position.x);
  let y = u32(position.y);
  let i = (x * resolution.y + y)*3;
  let oldColor = vec3f(image[i], image[i+1], image[i+2]);
  let newColor = mix(oldColor, color, 1.0/(min(f32(frame_count), f32(SAMPLE_FRAME))+1));
  image[i] = newColor.r;
  image[i+1] = newColor.g;
  image[i+2] = newColor.b;
  return vec4f(newColor, 1.0);
}
