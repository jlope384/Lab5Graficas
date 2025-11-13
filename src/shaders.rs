use nalgebra_glm::{Vec3, Vec4, Mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use nalgebra_glm as glm;
use std::sync::atomic::{AtomicUsize, Ordering};

static CURRENT_SHADER: AtomicUsize = AtomicUsize::new(0);

pub fn set_shader_index(idx: usize) {
  CURRENT_SHADER.store(idx, Ordering::Relaxed);
}

pub fn get_shader_index() -> usize {
  CURRENT_SHADER.load(Ordering::Relaxed)
}

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
  // Transform position
  let position = Vec4::new(
    vertex.position.x,
    vertex.position.y,
    vertex.position.z,
    1.0
  );
  let transformed = uniforms.model_matrix * position;

  // Perform perspective division
  let w = transformed.w;
  let transformed_position = Vec3::new(
    transformed.x / w,
    transformed.y / w,
    transformed.z / w
  );

  // Transform normal

  let model_mat3 = Mat3::new(
    uniforms.model_matrix[0], uniforms.model_matrix[1], uniforms.model_matrix[2],
    uniforms.model_matrix[4], uniforms.model_matrix[5], uniforms.model_matrix[6],
    uniforms.model_matrix[8], uniforms.model_matrix[9], uniforms.model_matrix[10]
  );
  let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

  let transformed_normal = normal_matrix * vertex.normal;

  // Create a new Vertex with transformed attributes
  Vertex {
    position: vertex.position,
    normal: vertex.normal,
    tex_coords: vertex.tex_coords,
    color: vertex.color,
    transformed_position,
    transformed_normal,
  }
}

/// Procedural planet shader that returns an RGB color (Vec3) with components in [0,1].
/// It combines several layers computed from position and normal only:
/// 1. Vertical gradient (poles lighter, equator darker) using abs(normal.y)
/// 2. Continental pattern: sin(pos.x*freq) * cos(pos.z*freq)
/// 3. Small trig-based noise for micro-variation
/// 4. Lambertian shading based on normal vs light direction
pub fn planet_shader(pos: Vec3, normal: Vec3) -> Vec3 {
  // normalize normal
  let n = normal.normalize();

  // Layer 1: vertical gradient (poles brighter)
  let pole = n.y.abs(); // 0 at equator, 1 at poles
  let gradient = 0.35 + 0.65 * pole; // range ~[0.35,1.0]

  // Layer 2: base continental / plate pattern (we'll reinterpret as tech plates)
  let freq = 0.12; // controls pattern scale
  let pattern = (pos.x * freq).sin() * (pos.z * freq).cos();
  // normalize pattern to [0,1]
  let continent = (pattern + 1.0) * 0.5;
  // smoothstep-like thresholding to create distinct plates
  let t = ((continent - 0.45) / 0.12).clamp(0.0, 1.0);
  let land_mask = t * t * (3.0 - 2.0 * t); // smoothstep

  // Layer 3: small trig-based noise (no external RNG)

  // Sci-fi palette: deep base, neon veins and bands
  let base_ocean = Vec3::new(0.02, 0.05, 0.08);
  let rock_dark = Vec3::new(0.10, 0.06, 0.16);
  let neon_cyan = Vec3::new(0.0, 0.95, 0.85);
  let neon_magenta = Vec3::new(0.95, 0.2, 0.85);

  // Mix base planet - plates (land_mask) produce darker tech-plates
  let base = base_ocean * (1.0 - land_mask) + rock_dark * land_mask;

  // Banding: wide flowing bands (like water belts or energy ribbons)
  let band_freq = 0.6;
  let flow = (pos.y * band_freq + pos.x * 0.02).sin();
  let band = (flow * 0.5 + 0.5).powf(1.5); // softer bands in [0,1]

  // Vein pattern: thin glowing lines using higher freq and sharpening
  let vein_freq = 3.5;
  let vein_raw = ((pos.x * vein_freq).sin() * (pos.z * vein_freq).sin()).abs();
  let vein = (vein_raw.powf(8.0)).clamp(0.0, 1.0); // sharp thin veins

  // Micro variations (subtle) - trig-based noise
  let noise = ((pos.x * 9.0).sin() * (pos.y * 13.0).cos() * (pos.z * 7.0).sin() + 1.0) * 0.5;
  let micro = (noise - 0.5) * 0.12;

  // Compose color: base + band modulation + micro variations
  let mut color = base * (1.0 + 0.45 * band) + Vec3::new(micro * 0.4, micro * 0.6, micro * 0.8);

  // Add neon veins glow where vein mask is strong
  color = color * (1.0 - vein * 0.7) + neon_cyan * (vein * 0.9) + neon_magenta * (band * 0.08);

  // Apply vertical gradient to change hue/intensity towards poles
  color *= 0.6 + 0.9 * gradient;

  // Lighting: basic lambert + specular-like highlight (sharp)
  let light_dir = Vec3::new(0.6, 0.7, 0.3).normalize();
  let lambert = glm::dot(&n, &light_dir).max(0.0);
  let spec = lambert.powf(60.0) * 1.4; // tight bright highlights
  let ambient = 0.18;
  let lit = ambient + 1.0 * lambert + spec;
  color *= lit;

  // Rim glow to accentuate silhouettes (using normal's view-approx)
  let rim = (1.0 - glm::dot(&n, &Vec3::new(0.0, 0.0, 1.0))).powf(2.0);
  color += neon_cyan * (rim * 0.18);

  // final clamp to [0,1]
  Vec3::new(color.x.clamp(0.0, 1.0), color.y.clamp(0.0, 1.0), color.z.clamp(0.0, 1.0))
}

/// Alternate planet shader variation (cooler palette)
pub fn planet_shader_cool(pos: Vec3, normal: Vec3) -> Vec3 {
  let mut c = planet_shader(pos, normal);
  // shift towards blue/cyan
  c = Vec3::new(c.x * 0.6, c.y * 0.9, (c.z * 1.1).min(1.0));
  c
}

/// Alternate planet shader variation (warm palette)
pub fn planet_shader_warm(pos: Vec3, normal: Vec3) -> Vec3 {
  let mut c = planet_shader(pos, normal);
  // shift towards warm/orange
  c = Vec3::new((c.x * 1.1).min(1.0), (c.y * 0.9).min(1.0), c.z * 0.6);
  c
}

/// Generic shade entry — dispatches to the selected shader variant.
pub fn shade(pos: Vec3, normal: Vec3) -> Vec3 {
  match get_shader_index() {
    1 => planet_shader_cool(pos, normal),
    2 => planet_shader_sun(pos, normal),
    _ => planet_shader(pos, normal),
  }
}

/// Sun-like shader: bright core, corona, and radial rays
pub fn planet_shader_sun(pos: Vec3, normal: Vec3) -> Vec3 {
  // Use the model-space position's length from origin to compute radial features.
  let r = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt();

  // Normalize normal for lighting
  let n = normal.normalize();

  // Core intensity: inverse falloff with smoothstep
  let core_radius = 40.0; // tune relative to model scale
  let core = (1.0 - (r / core_radius)).clamp(0.0, 1.0);
  let core = core.powf(2.5);

  // Corona: soft exponential falloff
  let corona = (- (r / (core_radius * 1.4))).exp();

  // Radial rays using angular pattern on spherical coords
  let theta = pos.y.atan2(pos.x); // angle in XY plane
  let phi = (pos.z / (r.max(1e-6))).acos();

  // Rays: a high-frequency angular modulation with sharpness
  let rays = ( (theta * 18.0).sin().abs().powf(6.0) * (1.0 - (phi / std::f32::consts::PI)).powf(2.0) ).clamp(0.0, 1.0);

  // Surface granulation (noise-like) from trig functions
  let micro = ((pos.x * 0.12).sin() * (pos.y * 0.13).cos() * (pos.z * 0.11).sin() * 0.25) + 0.75;

  // Base sun color (warm yellow-orange)
  let core_col = Vec3::new(1.0, 0.92, 0.5);
  let mid_col = Vec3::new(1.0, 0.6, 0.08);
  let outer_col = Vec3::new(0.9, 0.3, 0.05);

  // Combine layers
  let mut color = core_col * core + mid_col * (corona * 0.9) + outer_col * (corona * 0.5);

  // Add rays as bright streaks
  color = color + Vec3::new(1.0, 0.85, 0.6) * (rays * 0.9) * corona;

  // Modulate with micro detail
  color *= micro.clamp(0.8, 1.2);

  // Apply simple Lambert lighting for some shading
  let light_dir = Vec3::new(0.6, 0.7, 0.3).normalize();
  let lambert = glm::dot(&n, &light_dir).max(0.0) * 0.6 + 0.4; // keep it bright
  color *= lambert;

  // Add a soft rim/glow using normal vs view axis
  let rim = (1.0 - glm::dot(&n, &Vec3::new(0.0, 0.0, 1.0))).powf(3.0);
  color += Vec3::new(1.0, 0.7, 0.35) * (rim * 0.35);

  // Tone mapping / clamp
  Vec3::new(color.x.min(1.0), color.y.min(1.0), color.z.min(1.0))
}

