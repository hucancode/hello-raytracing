const PI = 3.141592653589793;
const PI2 = PI*2;
const EPSILON = 0.000001;
const FLT_MAX = 3.40282e+38;
const start_color = vec4(0.4, 0.7, 1.0, 1.0);
const end_color = vec4(0.0, 0.1, 0.3, 1.0);

@group(0) @binding(0)
var<uniform> resolution: vec2u;
@group(0) @binding(1)
var<uniform> time: u32;
@group(0) @binding(2)
var<storage, read_write> image: array<f32>;
@vertex
fn vs_main(@location(0) position: vec4f) -> @builtin(position) vec4f {
  return position;
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
  center: vec3f,
  radius: f32,
}
struct HitRecord {
  point: vec3f,
  normal: vec3f,
  t: f32,
}

fn point_on_ray(ray: Ray, t: f32) -> vec3f {
  return ray.origin + t * ray.direction;
}

fn rand1(seed: f32) -> f32 {
  return sin(seed) * 43758.5453123;
}
fn rand3(seed: vec3f) -> vec3f {
  return vec3f(rand1(seed.x), rand1(seed.y), rand1(seed.z));
}
fn random_on_hemisphere(seed: vec3f, normal: vec3f) -> vec3f {
  let t = f32(time);
  let v = normalize(rand3(seed));
  if(dot(v, normal) > 0) {
    return v;
  }
  return -v;
}

fn intersect_sphere(ray: Ray, sphere: Sphere) -> HitRecord {
    let oc = ray.origin - sphere.center;
    let a = dot(ray.direction, ray.direction);
    let b = 2 * dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius*sphere.radius;
    let discriminant = b*b - 4*a*c;
    if (discriminant < 0) {
        return HitRecord(vec3f(0), vec3f(0), -1);
    }
    let t = (-b - sqrt(discriminant) ) / (2*a);
    let hit_point = point_on_ray(ray, t);
    let normal = (hit_point - sphere.center) / sphere.radius;
    return HitRecord(hit_point, normal, t);
}

const RENDERED_FRAME = 200;
const SAMPLE_PER_FRAME = 10;
const BOUNCE_MAX = 20;
const LIGHT_COUNT = 1;
const OBJECT_COUNT = 4;
var<private> scene: array<Sphere, OBJECT_COUNT> = array(
  Sphere(vec3f(0, -10.5, -1), 10),
  Sphere(vec3f(0, 0, -1), 0.5),
  Sphere(vec3f(0.5, -0.3, -0.5), 0.1),
  Sphere(vec3f(-0.5, 0.1, -0.5), 0.1),
);

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
  let origin = vec3f(0);
  let focus_distance = 0.8f;
  let aspect_ratio = f32(resolution.x) / f32(resolution.y);
  var uv = position.xy / vec2f(f32(resolution.x - 1), f32(resolution.y - 1));
  uv = (2 * uv - vec2(1)) * vec2(aspect_ratio, -1);
  let direction = vec3(uv, -focus_distance);
  let ray = Ray(origin, direction);
  var color = vec3f(0);
  for (var i = 0; i < SAMPLE_PER_FRAME; i += 1) {
    color += trace(ray, f32(i))/f32(SAMPLE_PER_FRAME);
  }
  let x = u32(position.x);
  let y = u32(position.y);
  let i = (x * resolution.y + y)*3;
  let oldColor = vec3f(image[i], image[i+1], image[i+2]);
  let newColor = mix(oldColor, color, 1.0/f32(RENDERED_FRAME+1));
  image[i] = newColor.r;
  image[i+1] = newColor.g;
  image[i+2] = newColor.b;
  return vec4f(newColor, 1.0);
}

fn trace(ray: Ray, sample_idx: f32) -> vec3f {
  var ret = vec4f(0);
  var weight = 1.0;
  var current_ray = ray;
  for(var b = 0;b < BOUNCE_MAX; b++) {
    var closest_hit = HitRecord(vec3f(0), vec3f(0), FLT_MAX);
    for (var i = 0; i < OBJECT_COUNT; i += 1) {
      let hit = intersect_sphere(current_ray, scene[i]);
      if hit.t > 0 && hit.t < closest_hit.t {
        closest_hit = hit;
      }
    }
    if abs(closest_hit.t - FLT_MAX) < EPSILON {
      break;
    }
    let seed = vec3f(sample_idx * f32(time), sample_idx + sin(f32(time)), sample_idx + cos(f32(time)));
    let direction = random_on_hemisphere(seed, closest_hit.normal);
    let origin = closest_hit.point;
    current_ray = Ray(origin, direction);
    weight *= 0.5;
  }
  let x = ray.direction.y*0.5 + 0.5;
  let sky = mix(vec3f(1.0), vec3f(0.5,0.7,1.0), x);
  return weight*sky;
}
