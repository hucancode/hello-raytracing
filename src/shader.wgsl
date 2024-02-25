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
// @group(0) @binding(2)
// var<storage, read_write> image: array<array<array<f32, 3>>>;
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

fn sky_color(ray: Ray) -> vec4f {
  let t = 0.5 * (normalize(ray.direction).y + 1);
  return (1 - t) * vec4f(1) + t * vec4f(0.3, 0.5, 1.0, 1.0);
}

fn pcg3d(seed: vec3u) -> vec3u {
    var v = seed * 1664525u + 1013904223u;
    v.x += v.y*v.z; v.y += v.z*v.x; v.z += v.x*v.y;
    v ^= v >> vec3u(16);
    v.x += v.y*v.z; v.y += v.z*v.x; v.z += v.x*v.y;
    return v;
}
fn pcg3df(seed: vec3f) -> vec3f {
  return vec3f(pcg3d(vec3u(seed)));
}
fn rand11(seed: f32) -> f32 {
  return (sin(seed*0.876218)+cos(seed*0.22443)) * 43758.5453123;
}
fn rand22(seed: vec2f) -> vec2f {
  return vec2f(rand11(seed.x), rand11(seed.y));
}
fn rand33(seed: vec3f) -> vec3f {
  return vec3f(rand11(seed.x), rand11(seed.y), rand11(seed.z));
}
fn random_on_hemisphere(seed: vec3f, normal: vec3f) -> vec3f {
  let t = f32(time);
  let v = normalize(rand33(seed));
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
const SAMPLE_COUNT = 20;
const BOUNCE_MAX = 10;
const LIGHT_COUNT = 1;

const OBJECT_COUNT = 2;
var<private> scene: array<Sphere, OBJECT_COUNT> = array(
  Sphere(vec3f(0, -10.5, -1), 10),
  Sphere(vec3f(0, 0, -1), 0.5),
);

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
  let origin = vec3f(0);
  let focus_distance = 1f;
  let aspect_ratio = f32(resolution.x) / f32(resolution.y);
  var uv = position.xy / vec2f(f32(resolution.x - 1), f32(resolution.y - 1));
  uv = (2 * uv - vec2(1)) * vec2(aspect_ratio, -1);

  let direction = vec3(uv, -focus_distance);
  let ray = Ray(origin, direction);
  var ret = vec3f(0);
  for (var i = 0; i < SAMPLE_COUNT; i += 1) {
    ret += trace(ray, f32(i));
  }
  return vec4f(ret / f32(SAMPLE_COUNT), 1.0);
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
    let seed = vec3f(sample_idx+f32(time))+ray.direction + ray.origin;
    let direction = random_on_hemisphere(seed, closest_hit.normal);
    let origin = closest_hit.point;
    current_ray = Ray(origin, direction);
    weight *= 0.5;
  }
  let x = ray.direction.y*0.5 + 0.5;
  let sky = mix(vec3f(1.0), vec3f(0.5,0.7,1.0), x);
  return weight*sky;
}
