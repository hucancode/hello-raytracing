const PI = 3.141592653589793;
const PI2 = PI*2;
const EPSILON = 0.0001;
const FLT_MAX = 3.40282e+38;
const MAT_LAMBERTIAN = 1u;
const MAT_METAL = 2u;
const MAT_DIELECTRIC = 3u;
const SKY = vec3f(0.54, 0.86, 0.92);
const BLUE = vec3f(0.54, 0.7, 0.98);
const RED = vec3f(0.98, 0.2, 0.2);
const SAMPLE_FRAME = 1000;
const SAMPLE_PER_FRAME = 1;
const BOUNCE_MAX = 5;

@group(0) @binding(0)
var<uniform> resolution: vec2u;
@group(0) @binding(1)
var<uniform> frame_count: u32;
@group(0) @binding(2)
var<uniform> time: u32;
@group(0) @binding(3)
var<storage, read_write> image: array<f32>;
@group(0) @binding(4)
var<uniform> camera: Camera;
@group(1) @binding(0)
var<uniform> bvh_tree_size: vec2u;
@group(1) @binding(1)
var<storage> nodes: array<Node>;
@group(1) @binding(2)
var<storage> triangles: array<Triangle>;
@group(1) @binding(3)
var<storage> materials: array<Material>;

struct Camera {
  eye: vec4f,
  direction: vec4f,
  up: vec4f,
  right: vec4f,
  focal_length: f32,
  focal_blur_amount: f32,
  fov: f32,
}
struct Ray {
  origin: vec3f,
  direction: vec3f,
}
struct Node {
  bound_min: vec3f,
  bound_max: vec3f,
}
struct Triangle {
  a: vec4f,
  b: vec4f,
  c: vec4f,
  normal: vec3f,
  material: u32,
}
struct Material {
    albedo: vec4f,
    params: vec3f,
    id: u32,
}
struct HitRecord {
  point: vec3f,
  normal: vec3f,
  t: f32,
  material: Material,
  front_face: bool,
}

const DEFAULT_MATERIAL = Material(vec4f(0.0,0.4,0.0,1.0), vec3f(), MAT_LAMBERTIAN);
const EMPTY_HIT_RECORD = HitRecord(vec3f(), vec3f(), FLT_MAX, DEFAULT_MATERIAL, false);
const GRAY_MATERIAL = Material(vec4f(0.5,0.5,0.6,1.0), vec3f(), MAT_LAMBERTIAN);

@vertex
fn vs_main(@builtin(vertex_index) vertexIndex: u32) -> @builtin(position) vec4f {
  // 2-triangles screen space
  let a = vec4f(-1.0, -1.0, 0.0, 1.0);
  let b = vec4f(1.0, -1.0, 0.0, 1.0);
  let c = vec4f(1.0, 1.0, 0.0, 1.0);
  let d = vec4f(-1.0, 1.0, 0.0, 1.0);
  switch (vertexIndex) {
    case 0u, 3u: {
      return a;
    }
    case 1u: {
      return b;
    }
    case 2u, 4u: {
      return c;
    }
    case 5u, default: {
      return d;
    }
  }
  return vec4f(0.0, 0.0, 0.0, 1.0);
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
  direction = normalize(focus_point - origin);
  return Ray(origin.xyz, direction.xyz);
}

fn intersect_node(r: Ray, node: Node) -> bool {
  let size = (node.bound_max - node.bound_min).xyz;
  let ro = r.origin - node.bound_min.xyz;
  let d = 1.0/r.direction;
  let n = d*ro;
  let k = abs(d)*size;
  let t1 = -k - n;
  let t2 = k - n;
  let tmax = max(max(t1.x, t1.y), t1.z);
  let tmin = min(min(t2.x, t2.y), t2.z);
  return tmax <= tmin && tmin >= 0.0;
}

fn intersect_triangle(ray: Ray, i: u32, ret: ptr<function, HitRecord>) {
  let a = triangles[i].a.xyz;
  let b = triangles[i].b.xyz;
  let c = triangles[i].c.xyz;
  let normal = triangles[i].normal;
  let material = triangles[i].material;
  let ab = b - a;
  let ac = c - a;
  let p = cross(ray.direction, ac);
  let det = dot(ab, p);
  if abs(det) < EPSILON {
    return;
  }
  let ao = ray.origin - a;
  let inv_det = 1/det;
  let aoxab = cross(ao, ab);
  let u = inv_det * dot(ao, p);
  let v = inv_det * dot(ray.direction, aoxab);
  if (u < 0 || v < 0 || u + v > 1) {
    return;
  }
  let t = inv_det * dot(ac, aoxab);
  if (t < EPSILON || t > (*ret).t) {
    return;
  }
  (*ret).point = point_on_ray(ray, t);
  (*ret).normal = normal;
  (*ret).t = t;
  (*ret).material = materials[material];
  (*ret).front_face = dot((*ret).normal, ray.direction) > 0;
}

fn reflect(v: vec3f, n: vec3f) -> vec3f {
    return v - 2*dot(v,n)*n;
}
fn refract(uv: vec3f, n: vec3f, etai_over_etat: f32) -> vec3f {
    let cos_theta = min(dot(-uv, n), 1.0);
    let r_out_perp =  etai_over_etat * (uv + cos_theta*n);
    let len = length(r_out_perp);
    let r_out_parallel = -sqrt(abs(1.0 - len*len)) * n;
    return r_out_perp + r_out_parallel;
}
fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
    // Use Schlick's approximation for reflectance.
    var r0 = (1-ref_idx) / (1+ref_idx);
    r0 = r0*r0;
    return r0 + (1-r0)*pow((1 - cosine), 5.0);
}

// seems like the scatter function doesn't do well with triangles
fn scatter(state: ptr<function, u32>, ray: Ray, hit: HitRecord) -> Ray {
  switch hit.material.id {
    case MAT_LAMBERTIAN: {
      let direction = random_on_hemisphere(state, hit.normal);
      return Ray(hit.point, direction);
    }
    case MAT_METAL: {
      let fuzziness = hit.material.params.x;
      let direction = reflect(ray.direction, hit.normal) + fuzziness * random_on_hemisphere(state, hit.normal);
      return Ray(hit.point, normalize(direction));
    }
    case MAT_DIELECTRIC: {
      var ir = hit.material.params.x;
      if hit.front_face {
        ir = 1.0/ir;
      }
      let cos_theta = min(dot(-ray.direction, hit.normal), 1.0);
      let sin_theta = sqrt(1.0 - cos_theta*cos_theta);
      let cannot_refract = ir * sin_theta > 1.0;
      if (cannot_refract || reflectance(cos_theta, ir) > fract(rng_float(state))) {
          let direction = reflect(ray.direction, hit.normal);
          return Ray(hit.point, normalize(direction));
      } else {
          let direction = refract(ray.direction, hit.normal, ir);
          return Ray(hit.point, normalize(direction));
      }
    }
    default: {
      var ir = hit.material.params.x;
      if hit.front_face {
        ir = 1.0/ir;
      }
      let cos_theta = min(dot(-ray.direction, hit.normal), 1.0);
      let sin_theta = sqrt(1.0 - cos_theta*cos_theta);
      let cannot_refract = ir * sin_theta > 1.0;
      if (cannot_refract || reflectance(cos_theta, ir) > fract(rng_float(state))) {
          let direction = reflect(ray.direction, hit.normal);
          return Ray(hit.point, normalize(direction));
      } else {
          let direction = refract(ray.direction, hit.normal, ir);
          return Ray(hit.point, normalize(direction));
      }
      // return Ray(vec3f(), vec3f(0));
    }
  }
}
fn intersect_all_node(ray: Ray) -> HitRecord {
    var i = 1u;
    let n = bvh_tree_size.x;
    let m = bvh_tree_size.y;
    var ret = EMPTY_HIT_RECORD;
    var step = 0;
    while step < 1000 {
      step++;
      if i < n && intersect_node(ray, nodes[i]) {
        i *= 2u; // go to first children
        continue;
      }
      if i >= n {
        let j = i - n;
        if j >= m {
          break;
        }
        intersect_triangle(ray, j, &ret);
      }
      while (i&1u) == 1u {
        i /= 2u; // return to parent
      }
      if i == 0u {
        break;
      }
      i++; // go to next sibling
    }
    return ret;
}

fn trace(ray: Ray, state: ptr<function, u32>) -> vec3f {
  var attenuation = vec3f(1);
  var current_ray = ray;
  for(var b = 0;b < BOUNCE_MAX; b++) {
    var hit = intersect_all_node(current_ray);
    if abs(hit.t - FLT_MAX) < EPSILON {
      break;
    }
    current_ray = scatter(state, current_ray, hit);
    attenuation *= hit.material.albedo.rgb * 0.7;
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
  let i = (y * resolution.x + x)*3;
  let oldColor = vec3f(image[i], image[i+1], image[i+2]);
  let newColor = mix(oldColor, color, 1.0/(min(f32(frame_count), f32(SAMPLE_FRAME))+1));
  image[i] = newColor.r;
  image[i+1] = newColor.g;
  image[i+2] = newColor.b;
  return vec4f(newColor, 1.0);
}
