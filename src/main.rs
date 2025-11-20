use nalgebra_glm::{Mat4, Vec3};
use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::{Duration, Instant};

mod framebuffer;
mod triangle;
mod line;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;

use framebuffer::Framebuffer;
use obj::Obj;
use triangle::triangle;
use vertex::Vertex;
use shaders::{set_light_direction, set_light_intensity, set_noise_seed, set_shader_index, vertex_shader};

const DEFAULT_SCALE: f32 = 4.5;
const SOLAR_SYSTEM_SCALE: f32 = DEFAULT_SCALE * 0.15;

struct PlanetInstance {
    translation: Vec3,
    rotation: Vec3,
    scale: f32,
    shader_idx: usize,
    spin_speed: f32,
}

pub struct Uniforms {
    model_matrix: Mat4,
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, cos_x, -sin_x, 0.0,
        0.0, sin_x, cos_x, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y, 0.0, sin_y, 0.0,
        0.0, 1.0, 0.0, 0.0,
        -sin_y, 0.0, cos_y, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z, cos_z, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0, 0.0, translation.x,
        0.0, scale, 0.0, translation.y,
        0.0, 0.0, scale, translation.z,
        0.0, 0.0, 0.0, 1.0,
    );

    transform_matrix * rotation_matrix
}

fn rotate_vec3(vec: Vec3, rotation: Vec3) -> Vec3 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    // Apply X, then Y, then Z rotation so ordering matches create_model_matrix
    let mut rotated = vec;
    let y = rotated.y * cos_x - rotated.z * sin_x;
    let z = rotated.y * sin_x + rotated.z * cos_x;
    rotated.y = y;
    rotated.z = z;

    let x = rotated.x * cos_y + rotated.z * sin_y;
    let z = -rotated.x * sin_y + rotated.z * cos_y;
    rotated.x = x;
    rotated.z = z;

    let x = rotated.x * cos_z - rotated.y * sin_z;
    let y = rotated.x * sin_z + rotated.y * cos_z;
    rotated.x = x;
    rotated.y = y;

    rotated
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex]) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;
        if x < framebuffer.width && y < framebuffer.height {
            let color = fragment.color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn render_solar_system(
    framebuffer: &mut Framebuffer,
    vertex_array: &[Vertex],
    base_rotation: Vec3,
    camera_offset: Vec3,
    default_translation: Vec3,
    scale: f32,
    orbit_time: f32,
    solar_zoom: f32,
) {
    let view_offset = default_translation - camera_offset;
    let zoomed_scale = scale * solar_zoom;
    let scale_factor = zoomed_scale / DEFAULT_SCALE;
    let sun_scale = 8.0;
    let gas_scale = 5.0;
    let rock_scale = 3.2;
    let cheese_scale = 4.0;
    let cat_scale = 3.6;
    let bubble_scale = 4.3;
    let ice_scale = 4.8;
    let giant_scale = 6.2;
    let rock_radius = 190.0;
    let cat_radius = 320.0;
    let cheese_radius = 470.0;
    let gas_radius = 780.0;
    let bubble_radius = 1020.0;
    let ice_radius = 1350.0;
    let giant_radius = 1900.0;
    let rock_angle = orbit_time * 0.25;
    let cat_angle = orbit_time * 0.18;
    let cheese_angle = orbit_time * 0.12;
    let bubble_angle = orbit_time * 0.08;
    let gas_angle = orbit_time * 0.05;
    let ice_angle = orbit_time * 0.03;
    let giant_angle = orbit_time * 0.015;

    let planets = [
        PlanetInstance {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
            scale: sun_scale,
            shader_idx: 2,
            spin_speed: 0.0,
        },
        PlanetInstance {
            translation: Vec3::new(gas_radius * gas_angle.cos(), gas_radius * gas_angle.sin() * 0.65, 0.0),
            rotation: Vec3::new(0.05, 0.15, 0.0),
            scale: gas_scale,
            shader_idx: 0,
            spin_speed: 0.15,
        },
        PlanetInstance {
            translation: Vec3::new(rock_radius * rock_angle.cos(), rock_radius * rock_angle.sin() * 0.9, 0.0),
            rotation: Vec3::new(-0.08, 0.35, 0.0),
            scale: rock_scale,
            shader_idx: 1,
            spin_speed: 0.4,
        },
        PlanetInstance {
            translation: Vec3::new(cheese_radius * cheese_angle.cos(), cheese_radius * cheese_angle.sin() * 0.8, 0.0),
            rotation: Vec3::new(0.15, -0.22, 0.0),
            scale: cheese_scale,
            shader_idx: 3,
            spin_speed: 0.25,
        },
        PlanetInstance {
            translation: Vec3::new(cat_radius * cat_angle.cos(), cat_radius * cat_angle.sin() * 0.75, 0.0),
            rotation: Vec3::new(-0.12, 0.18, 0.05),
            scale: cat_scale,
            shader_idx: 4,
            spin_speed: 0.6,
        },
        PlanetInstance {
            translation: Vec3::new(bubble_radius * bubble_angle.cos(), bubble_radius * bubble_angle.sin() * 0.7, 0.0),
            rotation: Vec3::new(0.3, -0.1, 0.2),
            scale: bubble_scale,
            shader_idx: 5,
            spin_speed: 0.2,
        },
        PlanetInstance {
            translation: Vec3::new(ice_radius * ice_angle.cos(), ice_radius * ice_angle.sin() * 0.85, 0.0),
            rotation: Vec3::new(-0.05, 0.12, -0.08),
            scale: ice_scale,
            shader_idx: 6,
            spin_speed: 0.12,
        },
        PlanetInstance {
            translation: Vec3::new(giant_radius * giant_angle.cos(), giant_radius * giant_angle.sin() * 0.8, 0.0),
            rotation: Vec3::new(0.04, -0.18, 0.03),
            scale: giant_scale,
            shader_idx: 7,
            spin_speed: 0.08,
        },

    ];

    let sun_world = rotate_vec3(planets[0].translation, base_rotation);
    let sun_pulse = 0.85 + (orbit_time * 0.7).sin() * 0.15;

    for planet in planets.iter() {
        set_shader_index(planet.shader_idx);
        let planet_world = rotate_vec3(planet.translation, base_rotation);
        let rotated_translation = planet_world * solar_zoom;
        let mut light_vec = sun_world - planet_world;
        let distance = light_vec.magnitude();
        if distance < 1e-4 {
            light_vec = Vec3::new(0.0, 0.0, 1.0);
            set_light_direction(light_vec);
            set_light_intensity(1.1 * sun_pulse);
        } else {
            let light_dir = light_vec / distance;
            set_light_direction(light_dir);
            let falloff = 650.0;
            let attenuation = 1.0 / (1.0 + (distance / falloff).powi(2));
            let intensity = (0.25 + attenuation * 0.9) * sun_pulse;
            set_light_intensity(intensity);
        }

        let spin_angle = if planet.spin_speed.abs() > f32::EPSILON {
            orbit_time * planet.spin_speed
        } else {
            0.0
        };
        let spin_rotation = Vec3::new(0.0, spin_angle, 0.0);

        let model_matrix = create_model_matrix(
            rotated_translation + view_offset,
            planet.scale * scale_factor,
            base_rotation + planet.rotation + spin_rotation,
        );
        let uniforms = Uniforms { model_matrix };
        render(framebuffer, &uniforms, vertex_array);
    }

}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Rust Graphics - Renderer Example",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_position(500, 200);
    window.update();

    framebuffer.set_background_color(0x000000);

    if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        let nanos = (now.as_nanos() & 0xFFFF_FFFF) as u32;
        set_noise_seed(nanos);
    }

    let default_translation = Vec3::new(300.0, 200.0, 0.0);
    let mut camera_offset = Vec3::new(0.0, 0.0, 0.0);
    let mut rotation = Vec3::new(0.0, 0.0, 0.0);
    let mut scale = DEFAULT_SCALE * 0.15;
    let mut solar_zoom = 1.0;
    let mut solar_system_mode = true;

    let obj = Obj::load("assets/models/planetaff.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array();
    let start_time = Instant::now();

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        handle_input(
            &window,
            &mut camera_offset,
            &mut rotation,
            &mut scale,
            &mut solar_system_mode,
            &mut solar_zoom,
        );

        framebuffer.clear();

        if solar_system_mode {
            let orbit_time = start_time.elapsed().as_secs_f32();
            render_solar_system(
                &mut framebuffer,
                &vertex_arrays,
                rotation,
                camera_offset,
                default_translation,
                SOLAR_SYSTEM_SCALE,
                orbit_time,
                solar_zoom,
            );
        } else {
            set_light_direction(Vec3::new(0.6, 0.7, 0.3).normalize());
            set_light_intensity(1.0);
            let model_matrix = create_model_matrix(default_translation - camera_offset, scale, rotation);
            let uniforms = Uniforms { model_matrix };

            framebuffer.set_current_color(0xFFDDDD);
            render(&mut framebuffer, &uniforms, &vertex_arrays);
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}

fn handle_input(
    window: &Window,
    camera_offset: &mut Vec3,
    rotation: &mut Vec3,
    scale: &mut f32,
    solar_system_mode: &mut bool,
    solar_zoom: &mut f32,
) {
    // WASD-style translation (also keep arrow keys for convenience)
    if window.is_key_down(Key::Right) || window.is_key_down(Key::D) {
        camera_offset.x += 10.0;
    }
    if window.is_key_down(Key::Left) || window.is_key_down(Key::A) {
        camera_offset.x -= 10.0;
    }
    if window.is_key_down(Key::Up) || window.is_key_down(Key::W) {
        camera_offset.y -= 10.0;
    }
    if window.is_key_down(Key::Down) || window.is_key_down(Key::S) {
        camera_offset.y += 10.0;
    }
    if *solar_system_mode {
        if window.is_key_down(Key::Z) {
            *solar_zoom *= 1.08;
        }
        if window.is_key_down(Key::X) {
            *solar_zoom *= 0.92;
        }
    } else {
        if window.is_key_down(Key::Z) {
            *scale *= 1.08;
        }
        if window.is_key_down(Key::X) {
            *scale *= 0.92;
        }
    }
    if window.is_key_down(Key::Q) {
        rotation.x -= PI / 10.0;
    }
    if window.is_key_down(Key::U) {
        rotation.x += PI / 10.0;
    }
    if window.is_key_down(Key::E) {
        rotation.y -= PI / 10.0;
    }
    if window.is_key_down(Key::R) {
        rotation.y += PI / 10.0;
    }
    if window.is_key_down(Key::T) {
        rotation.z -= PI / 10.0;
    }
    if window.is_key_down(Key::Y) {
        rotation.z += PI / 10.0;
    }

    if window.is_key_down(Key::Key1) {
        *solar_system_mode = true;
    }
    if window.is_key_down(Key::Key2) {
        *solar_system_mode = false;
        set_shader_index(1);
    }
    if window.is_key_down(Key::Key3) {
        *solar_system_mode = false;
        set_shader_index(2);
    }
    if window.is_key_down(Key::Key4) {
        *solar_system_mode = false;
        set_shader_index(0);
    }
    if window.is_key_down(Key::Key5) {
        *solar_system_mode = false;
        set_shader_index(3);
    }
    if window.is_key_down(Key::Key6) {
        *solar_system_mode = false;
        set_shader_index(4);
    }
    if window.is_key_down(Key::Key7) {
        *solar_system_mode = false;
        set_shader_index(5);
    }
    if window.is_key_down(Key::Key8) {
        *solar_system_mode = false;
        set_shader_index(6);
    }
    if window.is_key_down(Key::Key9) {
        *solar_system_mode = false;
        set_shader_index(7);
    }
}
