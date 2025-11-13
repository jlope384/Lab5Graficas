use nalgebra_glm::{Vec3, Vec4, Mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use nalgebra_glm as glm;
use std::sync::atomic::{AtomicUsize, AtomicU32, Ordering};

static CURRENT_SHADER: AtomicUsize = AtomicUsize::new(0);
static NOISE_SEED: AtomicU32 = AtomicU32::new(0);

pub fn set_shader_index(idx: usize) {
  CURRENT_SHADER.store(idx, Ordering::Relaxed);
}

pub fn get_shader_index() -> usize {
  CURRENT_SHADER.load(Ordering::Relaxed)
}

pub fn set_noise_seed(seed: u32) {
  NOISE_SEED.store(seed, Ordering::Relaxed);
}

fn get_noise_seed() -> u32 {
  NOISE_SEED.load(Ordering::Relaxed)
}

fn noise_seed_vec3() -> Vec3 {
  let s = get_noise_seed() as f32;
  // Pseudo-random generation via sin/fract trick
  let r1 = ((s * 0.12345).sin() * 43758.5453).fract();
  let r2 = ((s * 0.34567).sin() * 28123.1234).fract();
  let r3 = ((s * 0.78901).sin() * 15937.9876).fract();
  // Map [0,1) -> [-1,1]
  Vec3::new(r1 * 2.0 - 1.0, r2 * 2.0 - 1.0, r3 * 2.0 - 1.0)
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

/// Gaseous planet shader: banded clouds, flow-warp and soft lighting
pub fn planet_shader_gas(pos: Vec3, normal: Vec3) -> Vec3 {
  let n = normal.normalize();

  // Directions for isotropic trig-noise and domain warp
  let v1 = Vec3::new(0.36, 0.93, 0.04).normalize();
  let v2 = Vec3::new(0.79, -0.61, 0.08).normalize();
  let v3 = Vec3::new(-0.49, 0.12, 0.86).normalize();

  // Base band coordinate (latitude-like using normal.y) with domain warp
  let mut band = n.y * 14.0; // number of bands
  let warp = (glm::dot(&pos, &v1) * 0.9).sin() * 0.55
          + (glm::dot(&pos, &v2) * 1.6).sin() * 0.35
          + (glm::dot(&pos, &v3) * 2.3).sin() * 0.18;
  band += warp;

  // Banded pattern (0..1) with gentle sharpening
  let band_val = (band.sin() * 0.5 + 0.5).powf(1.2);

  // Fine streaking along flow
  let streak = ((glm::dot(&pos, &v2) * 6.0).sin().abs() * 0.25)
             + ((glm::dot(&pos, &v3) * 8.5).sin().abs() * 0.15);

  // Softer gaseous palette (pastel creams/tans/ochres/blue-grays)
  let cream     = Vec3::new(0.92, 0.88, 0.80);
  let tan       = Vec3::new(0.78, 0.66, 0.50);
  let ochre     = Vec3::new(0.76, 0.58, 0.30);
  let blue_gray = Vec3::new(0.65, 0.72, 0.80);

  // Two base band families and a slow alternation between families
  let band_family_a = cream * (1.0 - band_val) + tan * band_val;       // light bands
  let band_family_b = ochre * (1.0 - band_val) + blue_gray * band_val; // darker/cooler bands
  let family_alt = ((glm::dot(&pos, &v1) * 0.25 + n.y * 0.6).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
  let family_mix = (family_alt * 0.6 + 0.2).clamp(0.0, 1.0); // mostly A, sometimes B

  let mut color = band_family_a * (1.0 - family_mix) + band_family_b * family_mix;
  // Add gentle streak modulation
  color = color * (1.0 + 0.18 * streak);

  // Subtle turbulence to break uniformity
  let turb = ((glm::dot(&pos, &v1) * 3.1).sin().abs() * 0.12)
           + ((glm::dot(&pos, &v2) * 4.7).sin().abs() * 0.08);
  color *= 1.0 + turb;

  // Soft lighting (clouds): mostly diffuse, low specular
  let light_dir = Vec3::new(0.6, 0.7, 0.3).normalize();
  let lambert = glm::dot(&n, &light_dir).max(0.0);
  let spec = lambert.powf(8.0) * 0.05;
  let ambient = 0.35;
  let lit = ambient + 0.7 * lambert + spec;
  color *= lit;

  // Gentle rim light to suggest atmospheric scattering
  let rim = (1.0 - glm::dot(&n, &Vec3::new(0.0, 0.0, 1.0))).powf(2.2);
  color += Vec3::new(0.12, 0.18, 0.24) * (rim * 0.18);

  Vec3::new(color.x.clamp(0.0, 1.0), color.y.clamp(0.0, 1.0), color.z.clamp(0.0, 1.0))
}

/// Rocky planet shader: stratified rock, regolith and cracks with lambertian lighting
pub fn planet_shader_rock(pos: Vec3, normal: Vec3) -> Vec3 {
  let n = normal.normalize();
  let seed_vec = noise_seed_vec3();
  // Seeded position for noise domain warp (keeps shape, changes patterns)
  let p = pos + seed_vec * 12.3;

  // Isotropic trig-noise helpers
  let v1 = Vec3::new(0.36, 0.93, 0.04).normalize();
  let v2 = Vec3::new(0.79, -0.61, 0.08).normalize();
  let v3 = Vec3::new(-0.49, 0.12, 0.86).normalize();

  let f1 = (glm::dot(&p, &v1) * 0.25).sin();
  let f2 = (glm::dot(&p, &v2) * 0.55).sin();
  let f3 = (glm::dot(&p, &v3) * 1.10).sin();
  let noise_base = (0.55 * f1 + 0.3 * f2 + 0.15 * f3) * 0.5 + 0.5; // [0,1]

  // Multi-frequency for height/roughness
  let f4 = (glm::dot(&p, &v1) * 2.0).sin().abs();
  let f5 = (glm::dot(&p, &v2) * 3.3).sin().abs();
  let f6 = (glm::dot(&p, &v3) * 5.1).sin().abs();
  let height = (0.5 * noise_base + 0.3 * f4 + 0.2 * (0.5 * f5 + 0.5 * f6)).clamp(0.0, 1.0);

  // Strata bands along a direction
  let sdir = (v1 + v2 * 0.3 + v3 * 0.2 + seed_vec * 0.2).normalize();
  let strata_raw = (glm::dot(&p, &sdir) * 0.6).sin().abs();
  let strata = strata_raw.powf(3.0); // thin bands

  // Crack network (thin dark lines)
  let crack_a = ((glm::dot(&p, &v1) * 6.5).sin() * (glm::dot(&p, &v2) * 6.2).sin()).abs();
  let crack_b = ((glm::dot(&p, &v2) * 7.1).sin() * (glm::dot(&p, &v3) * 7.4).sin()).abs();
  let cracks = (crack_a.min(crack_b)).powf(12.0).clamp(0.0, 1.0);

  // Base rocky palette
  let basalt = Vec3::new(0.12, 0.10, 0.09);
  let regolith = Vec3::new(0.38, 0.31, 0.22);
  let iron_oxide = Vec3::new(0.55, 0.32, 0.15);

  // Blend materials (add slope-based dust accumulation)
  let up = if pos.magnitude() > 0.0 { pos / pos.magnitude() } else { Vec3::new(0.0, 1.0, 0.0) };
  let slope = (1.0 - glm::dot(&n, &up)).clamp(0.0, 1.0); // steep -> 1, flat -> 0
  let dust_mask = (height * 0.7 + strata * 0.5 + (1.0 - slope) * 0.6).clamp(0.0, 1.0);
  let iron_mask = ((noise_base - 0.6) / 0.25).clamp(0.0, 1.0);
  let mut albedo = basalt * (1.0 - dust_mask) + regolith * dust_mask;
  albedo = albedo * (1.0 - iron_mask) + iron_oxide * iron_mask;

  // Apply cracks as dark lines (subtractive)
  albedo *= 1.0 - (cracks * 0.6);

  // Micro roughness modulation
  let micro = ((glm::dot(&p, &v1) * 9.0).sin() * (glm::dot(&p, &v2) * 11.0).cos()).abs() * 0.2;
  let mut color = albedo * (1.0 - 0.15) + albedo * micro;

  // Ambient occlusion-like darkening using (1 - height)
  let ao = (1.0 - height).clamp(0.0, 1.0);
  color *= 1.0 - 0.35 * ao;

  // Procedural craters (sparse), using cell hash and spherical distance
  let cscale = 0.06; // crater density; higher -> fewer cells per unit
  let cx = (p.x * cscale).floor();
  let cy = (p.y * cscale).floor();
  let cz = (p.z * cscale).floor();
  let cell = Vec3::new(cx, cy, cz);
  // Hash helpers to get pseudo-random in [0,1)
  let h1 = {
    let d = glm::dot(&cell, &Vec3::new(12.9898, 78.233, 37.719)) + seed_vec.x * 97.0;
    let s = (d).sin() * 43758.5453;
    s - s.floor()
  };
  let h2 = {
    let d = glm::dot(&cell, &Vec3::new(93.989, 67.345, 24.123)) + seed_vec.y * 73.0;
    let s = (d).sin() * 12753.5453;
    s - s.floor()
  };
  let h3 = {
    let d = glm::dot(&cell, &Vec3::new(53.786, 12.345, 91.532)) + seed_vec.z * 59.0;
    let s = (d).sin() * 31837.1234;
    s - s.floor()
  };
  // Only place a crater in some cells
  if h1 > 0.88 {
    let off = Vec3::new(h1 - 0.5, h2 - 0.5, h3 - 0.5) * (1.0 / cscale);
    let center = (cell / cscale) + off;
    let pn = if pos.magnitude() > 0.0 { pos / pos.magnitude() } else { n };
    let cn = if center.magnitude() > 0.0 { center / center.magnitude() } else { n };
    let ang = (glm::dot(&pn, &cn)).clamp(-1.0, 1.0).acos(); // radians
    let w = 0.045 + h2 * 0.02; // crater angular radius
    let t = (1.0 - (ang / w)).clamp(0.0, 1.0);
    let bowl = t * t; // inside darkening
    let rim = (1.0 - ((ang - w * 0.85).abs() / (w * 0.25)).clamp(0.0, 1.0)).powf(4.0);
    let crater_dark = bowl * 0.22;
    let rim_bright = rim * 0.08;
    color *= 1.0 - crater_dark;
    color += Vec3::new(0.25, 0.22, 0.18) * rim_bright; // slightly warmer rim
  }

  // Lighting: rough rock, low specular
  let light_dir = Vec3::new(0.6, 0.7, 0.3).normalize();
  let lambert = glm::dot(&n, &light_dir).max(0.0);
  let spec = lambert.powf(12.0) * 0.15; // rough highlight
  let ambient = 0.22;
  let lit = ambient + 0.95 * lambert + spec;
  color *= lit;

  Vec3::new(color.x.clamp(0.0, 1.0), color.y.clamp(0.0, 1.0), color.z.clamp(0.0, 1.0))
}

/// Generic shade entry — dispatches to the selected shader variant.
pub fn shade(pos: Vec3, normal: Vec3) -> Vec3 {
  match get_shader_index() {
    0 => planet_shader_gas(pos, normal),
    1 => planet_shader_rock(pos, normal),
    2 => planet_shader_sun(pos, normal),
    _ => planet_shader_gas(pos, normal),
  }
}

/// Sun-like shader: bright core, corona, and radial rays
pub fn planet_shader_sun(pos: Vec3, normal: Vec3) -> Vec3 {
  // Normalize normal for view-dependent effects
  let n = normal.normalize();

  // Uniform emissive base: warm orange, slightly less bright overall
  let base = Vec3::new(1.0, 0.65, 0.18);
  let mut color = base * 0.85; // tone down brightness a bit

  // Isotropic turbulence (replaces angular rays to avoid vertical lines)
  let p = pos;
  let v1 = Vec3::new(0.36, 0.93, 0.04).normalize();
  let v2 = Vec3::new(0.79, -0.61, 0.08).normalize();
  let v3 = Vec3::new(-0.49, 0.12, 0.86).normalize();
  let n1 = (glm::dot(&p, &v1) * 0.9).sin();
  let n2 = (glm::dot(&p, &v2) * 1.6).sin();
  let n3 = (glm::dot(&p, &v3) * 2.3).sin();
  let turb = (n1.abs() * 0.5 + n2.abs() * 0.3 + n3.abs() * 0.2).clamp(0.0, 1.0);
  color += Vec3::new(1.0, 0.8, 0.45) * (turb * 0.25);

  // Gentle additive flicker (kept subtle)
  let flicker = ((pos.x * 0.12).sin() * (pos.y * 0.13).cos() * (pos.z * 0.11).sin() * 0.10 + 0.10).max(0.0);
  color += base * flicker;

  // Procedural granulation (isotropic, avoids axis-aligned banding)
  let g1 = (glm::dot(&p, &v1) * 0.8).sin().abs();
  let g2 = (glm::dot(&p, &v2) * 1.2).sin().abs();
  let g3 = (glm::dot(&p, &v3) * 1.6).sin().abs();
  let gran = (0.5 * g1 + 0.3 * g2 + 0.2 * g3).clamp(0.0, 1.0);
  // Center around 1.0 with small variance: 0.85..1.10
  let gran_amp = 0.25; // how much granulation affects
  color *= (1.0 - gran_amp * 0.6) + gran_amp * gran; // mostly small dark/light patches

  // Sunspots: higher frequency and gentler darkening to avoid large black areas
  let n_low = ((pos.x * 0.08).sin() * (pos.y * 0.075).cos() * (pos.z * 0.065).sin() + 1.0) * 0.5; // [0,1]
  // Invert and sharpen to get smaller spot islands
  let t = ((0.50 - n_low) / 0.15).clamp(0.0, 1.0);
  let mut spots = t * t * (3.0 - 2.0 * t); // smoothstep
  spots = spots.powf(2.2); // smaller, tighter cores
  // Apply penumbra/umbra effect: lighter overall
  let penumbra = spots * 0.18;
  let umbra = spots.powf(1.6) * 0.18; // core
  let mut spot_att = 1.0 - (penumbra + umbra);
  spot_att = spot_att.max(0.55); // brightness floor: never below 55%
  color *= spot_att; // multiplicative darkening in spots only

  // Add a soft rim/glow using normal vs view axis (additive only)
  let rim = (1.0 - glm::dot(&n, &Vec3::new(0.0, 0.0, 1.0))).powf(3.0);
  color += Vec3::new(1.0, 0.6, 0.25) * (rim * 0.2); // keep rim subtler for "less bright"

  // Tone mapping / clamp
  Vec3::new(color.x.min(1.0), color.y.min(1.0), color.z.min(1.0))
}

