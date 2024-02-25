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
  material: Material,
}
struct HitRecord {
  point: vec3f,
  normal: vec3f,
  t: f32,
  material: Material,
}
struct Material {
    id: i32,
    albedo: vec3f,
    param1: f32,
}


const PI = 3.141592653589793;
const PI2 = PI*2;
const EPSILON = 0.000001;
const FLT_MAX = 3.40282e+38;
const start_color = vec4(0.4, 0.7, 1.0, 1.0);
const end_color = vec4(0.0, 0.1, 0.3, 1.0);

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

fn rng_vec(state: ptr<function, u32>) -> vec3f {
    return vec3f(rng_float(state), rng_float(state), rng_float(state));
}

fn point_on_ray(ray: Ray, t: f32) -> vec3f {
  return ray.origin + t * ray.direction;
}
fn random_on_hemisphere(state: ptr<function, u32>, normal: vec3f) -> vec3f {
  let t = f32(time);
  let v = normalize(rng_vec(state));
  if length(v) < EPSILON {
    return normal;
  }
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
      return HitRecord(vec3f(0), vec3f(0), -1, sphere.material);
  }
  let t = (-b - sqrt(discriminant) ) / (2*a);
  let hit_point = point_on_ray(ray, t);
  let normal = (hit_point - sphere.center) / sphere.radius;
  return HitRecord(hit_point, normal, t, sphere.material);
}
fn reflect(v: vec3f, n: vec3f) -> vec3f {
    return v - 2*dot(v,n)*n;
}
fn scatter(state: ptr<function, u32>, ray: Ray, hit: HitRecord) -> Ray {
  switch hit.material.id {
    case MAT_LAMBERTIAN: {
      let direction = random_on_hemisphere(state, hit.normal);
      return Ray(hit.point, direction);
    }
    case MAT_METAL: {
      let fuzziness = hit.material.param1;
      let direction = reflect(normalize(ray.direction), hit.normal) + fuzziness * random_on_hemisphere(state, hit.normal);
      return Ray(hit.point, normalize(direction));
    }
    case MAT_DIELECTRIC: {
      
    }
    default: {
      
    }
  }
  return Ray(vec3f(0), vec3f(0));
}

const MAT_LAMBERTIAN = 1;
const MAT_METAL = 2;
const MAT_DIELECTRIC = 3;

const RENDERED_FRAME = 10;
const SAMPLE_PER_FRAME = 5;
const BOUNCE_MAX = 20;
const LIGHT_COUNT = 1;
const OBJECT_COUNT = 4;
var<private> scene: array<Sphere, OBJECT_COUNT> = array(
  Sphere(
    vec3f(0, -10.5, -1), 
    10, 
    Material(
      MAT_LAMBERTIAN,
      vec3f(0.8, 0.8, 0.0),
      0.0,
    )
  ),
  Sphere(
    vec3f(-1, 0, -1), 
    0.5,
    Material(
      MAT_METAL,
      vec3f(0.9, 0.15, 0.02),
      0.9,
    )
  ),
  Sphere(
    vec3f(0, 0, -1), 
    0.5,
    Material(
      MAT_LAMBERTIAN,
      vec3f(0.2, 0.8, 0.2),
      0.0,
    )
  ),
  Sphere(
    vec3f(1, 0, -1), 
    0.5,
    Material(
      MAT_METAL,
      vec3f(0.1, 0.4, 0.9),
      0.1,
    )
  ),
);
@fragment
fn test_rng(@builtin(position) position: vec4f) -> @location(0) vec4f {
  var state = u32(position.x)*resolution.y + u32(position.y);
  return vec4f(rng_vec(&state), 1.0);
}
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
  var rng_state = u32(position.x)*resolution.y + u32(position.y) * time;
  for (var i = 0; i < SAMPLE_PER_FRAME; i += 1) {
    color += trace(ray, &rng_state)/f32(SAMPLE_PER_FRAME);
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

fn trace(ray: Ray, state: ptr<function, u32>) -> vec3f {
  var ret = vec4f(0);
  var attenuation = vec3f(1);
  var current_ray = ray;
  var first_hit = true;
  for(var b = 0;b < BOUNCE_MAX; b++) {
    var closest_hit = HitRecord(
      vec3f(0), 
      vec3f(0), 
      FLT_MAX, 
      Material(
        MAT_METAL,
        vec3f(0.8, 0.6, 0.2),
        0.8,
      )
    );
    for (var i = 0; i < OBJECT_COUNT; i += 1) {
      let hit = intersect_sphere(current_ray, scene[i]);
      if hit.t > 0 && hit.t < closest_hit.t {
        closest_hit = hit;
      }
    }
    if abs(closest_hit.t - FLT_MAX) < EPSILON {
      break;
    }
    current_ray = scatter(state, current_ray, closest_hit);
    if first_hit {
      attenuation = closest_hit.material.albedo;
      first_hit = false;
    } else {
      attenuation *= closest_hit.material.albedo;
    }
  }
  let x = ray.direction.y*0.5 + 0.5;
  let sky = mix(vec3f(1.0), vec3f(0.5,0.7,1.0), x);
  return attenuation * sky;
}
