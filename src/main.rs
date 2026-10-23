mod color;
mod framebuffer;
mod line;
mod obj;
mod triangle;
mod vertex;

use color::Color;
use framebuffer::Framebuffer;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use nalgebra_glm::{self as glm, Mat4, Vec3};
use obj::{Face, load_obj, setup_vertex_array};
use triangle::{fill_triangle, triangle};
use vertex::print_vertex;

const SCREEN_WIDTH: usize = 960;
const SCREEN_HEIGHT: usize = 720;
const MODEL_PATH: &str = "naveblender.obj";
const RENDER_PATH: &str = "render.png";


const FIT: f32 = 0.52;
const LIGHT_DIR: [f32; 3] = [-0.35, -0.65, 0.7];
const AMBIENT: f32 = 0.28;
const SELECTED_COLOR: Color = Color::new(0, 230, 255);

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Relleno,
    RellenoAristas,
    Alambre,
}

impl Mode {
    fn next(self) -> Self {
        match self {
            Mode::Relleno => Mode::RellenoAristas,
            Mode::RellenoAristas => Mode::Alambre,
            Mode::Alambre => Mode::Relleno,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Mode::Relleno => "Relleno",
            Mode::RellenoAristas => "Relleno + aristas",
            Mode::Alambre => "Alambre",
        }
    }
}

struct View {
    yaw: f32,
    pitch: f32,
    zoom: f32,
}

impl Default for View {
    fn default() -> Self {
        View { yaw: -0.85, pitch: 0.5, zoom: 1.0 }
    }
}

struct Settings {
    mode: Mode,
    selected: Option<usize>,
}

struct Scene {
    vertex_array: Vec<Vec3>,
    faces: Vec<Face>,
    center: Vec3,
    radius: f32,
}

/// las estrellas.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }

    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// Colores por pieza de la nave 
fn part_color(object: &str) -> Color {
    const ORANGE: Color = Color::new(255, 120, 20);
    const BLACK: Color = Color::new(38, 36, 42);
    const GRAY: Color = Color::new(160, 160, 170);
    const PURPLE: Color = Color::new(125, 55, 210);

    match object {
        // Placas superiores de alas 
        "Cube.013" | "Cube.014" | "Cube.015" | "Cube.016" => ORANGE,
        // Placas intermedias y la punta inferior
        "Cube.018" | "Cube.019" | "Cube.020" | "Cube.022" => BLACK,
        // Placas inferiores
        "Cube.021" | "Cube.023" => PURPLE,
        // Cabina
        "Cube.017" => BLACK,
        // Cuerpo central
        "Cube.008" => PURPLE,
        // Cápsulas laterales
        "Cube.004" | "Cube.012" => GRAY,
        // Aleta de cola
        "Cube.001" | "Cube.006" | "Cube.007" => GRAY,
        // Aletas de las cápsulas
        "Cube.002" | "Cube.003" | "Cube.005" | "Cube.009" | "Cube.010" | "Cube.011" => PURPLE,
        // Motores
        name if name.starts_with("Cylinder") => GRAY,
        _ => ORANGE,
    }
}

fn model_matrix(scene: &Scene, view: &View, framebuffer: &Framebuffer) -> Mat4 {
    let scale =
        view.zoom * FIT * framebuffer.width.min(framebuffer.height) as f32 / scene.radius;
    let screen_center = glm::vec3(
        framebuffer.width as f32 / 2.0,
        framebuffer.height as f32 / 2.0,
        0.0,
    );

    // Centrar el modelo en el origen
    let matrix = glm::translation(&screen_center)
        * glm::scaling(&glm::vec3(scale, -scale, scale))
        * glm::rotation(view.pitch, &glm::vec3(1.0, 0.0, 0.0))
        * glm::rotation(view.yaw, &glm::vec3(0.0, 1.0, 0.0))
        * glm::translation(&-scene.center);

    let (mut min_x, mut max_x, mut min_y, mut max_y) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for v in &scene.vertex_array {
        let p = transform(&matrix, v);
        min_x = min_x.min(p.x);
        max_x = max_x.max(p.x);
        min_y = min_y.min(p.y);
        max_y = max_y.max(p.y);
    }
    let offset = glm::vec3(
        screen_center.x - (min_x + max_x) / 2.0,
        screen_center.y - (min_y + max_y) / 2.0,
        0.0,
    );
    glm::translation(&offset) * matrix
}

fn transform(matrix: &Mat4, v: &Vec3) -> Vec3 {
    let r = matrix * glm::vec4(v.x, v.y, v.z, 1.0);
    glm::vec3(r.x, r.y, r.z)
}

fn render(framebuffer: &mut Framebuffer, scene: &Scene, view: &View, settings: &Settings) {
    let matrix = model_matrix(scene, view, framebuffer);
    let light = glm::normalize(&glm::vec3(LIGHT_DIR[0], LIGHT_DIR[1], LIGHT_DIR[2]));

    let transformed: Vec<Vec3> = scene
        .vertex_array
        .iter()
        .map(|v| transform(&matrix, v))
        .collect();

    framebuffer.depth_test = settings.mode != Mode::Alambre;

    // Cada 3 vértices del arreglo son un triángulo.
    for (i, tri) in transformed.chunks_exact(3).enumerate() {
        let (a, b, c) = (&tri[0], &tri[1], &tri[2]);
        let base = if settings.selected == Some(i) {
            SELECTED_COLOR
        } else {
            part_color(&scene.faces[i].object)
        };
        framebuffer.set_current_id(i as i32);

        if settings.mode == Mode::Alambre {
            framebuffer.set_current_color(base);
            triangle(framebuffer, a, b, c);
            continue;
        }

        // Sombreado plano: normal de la cara orientada hacia la cámara (+z).
        let normal = glm::cross(&(b - a), &(c - a));
        let length = glm::length(&normal);
        let intensity = if length > 1e-9 {
            let mut n = normal / length;
            if n.z < 0.0 {
                n = -n;
            }
            AMBIENT + (1.0 - AMBIENT) * glm::dot(&n, &light).max(0.0)
        } else {
            AMBIENT
        };

        framebuffer.set_current_color(base.scale(intensity));
        fill_triangle(framebuffer, a, b, c);

        if settings.mode == Mode::RellenoAristas {
            let lift = glm::vec3(0.0, 0.0, 0.5); // evita que el relleno tape la arista
            framebuffer.set_current_color(base.scale(intensity * 0.45));
            triangle(framebuffer, &(a + lift), &(b + lift), &(c + lift));
        }
    }
}

fn starfield(width: usize, height: usize) -> Vec<u32> {
    let top = Color::new(8, 10, 26);
    let bottom = Color::new(22, 14, 40);
    let mut pixels: Vec<u32> = (0..width * height)
        .map(|i| top.lerp(&bottom, (i / width) as f32 / height as f32).to_hex())
        .collect();

    let mut rng = Rng::new(42);
    for _ in 0..(width * height / 2500) {
        let x = (rng.next_f32() * width as f32) as usize;
        let y = (rng.next_f32() * height as f32) as usize;
        let b = (90.0 + rng.next_f32() * 165.0) as u8;
        pixels[y * width + x] = Color::new(b, b, b.saturating_add(20)).to_hex();
    }
    pixels
}

fn describe_triangle(scene: &Scene, i: usize) {
    let face = &scene.faces[i];
    let indices: Vec<i32> = face.vertex_indices.iter().map(|idx| idx[0]).collect();
    println!("Triángulo #{i} | objeto {} | índices de vértice (base 0): {indices:?}", face.object);
    for (k, v) in scene.vertex_array[i * 3..i * 3 + 3].iter().enumerate() {
        print_vertex(&format!("  v{k}"), v);
    }
}

fn window_title(settings: &Settings) -> String {
    let selected = match settings.selected {
        Some(i) => format!("Triángulo #{i}"),
        None => "Clic en un triángulo para ver su índice".to_string(),
    };
    format!(
        "Software Renderer | Modo: {} | {} | Mouse/Flechas: Rotar | Rueda/Q/E: Zoom | R: Reiniciar | SPACE: Modo | P: Captura",
        settings.mode.name(),
        selected
    )
}

fn main() {
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    if !load_obj(MODEL_PATH, &mut vertices, &mut faces) {
        eprintln!("No se pudo cargar el modelo {MODEL_PATH}");
        std::process::exit(1);
    }

    let vertex_array = setup_vertex_array(&vertices, &faces);
    println!(
        "{MODEL_PATH}: {} vértices, {} caras -> {} vértices en el arreglo",
        vertices.len(),
        faces.len(),
        vertex_array.len()
    );

    let min = vertices.iter().fold(vertices[0], |m, v| glm::min2(&m, v));
    let max = vertices.iter().fold(vertices[0], |m, v| glm::max2(&m, v));
    let scene = Scene {
        vertex_array,
        faces,
        center: (min + max) / 2.0,
        radius: glm::length(&(max - min)) / 2.0,
    };

    let mut framebuffer = Framebuffer::new(SCREEN_WIDTH, SCREEN_HEIGHT);
    framebuffer.set_background(starfield(SCREEN_WIDTH, SCREEN_HEIGHT));

    let mut view = View::default();
    let mut settings = Settings { mode: Mode::Relleno, selected: None };

    // `cargo run --release -- --png` genera render.png sin abrir ventana.
    if std::env::args().any(|arg| arg == "--png") {
        framebuffer.clear();
        render(&mut framebuffer, &scene, &view, &settings);
        match framebuffer.save_png(RENDER_PATH) {
            Ok(()) => println!("Imagen guardada en {RENDER_PATH}"),
            Err(e) => eprintln!("No se pudo guardar {RENDER_PATH}: {e}"),
        }
        return;
    }

    let mut title = window_title(&settings);
    let mut window = Window::new(&title, SCREEN_WIDTH, SCREEN_HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| panic!("No se pudo abrir la ventana: {e}"));
    window.set_target_fps(60);

    println!("Controles: arrastrar el mouse o flechas = rotar | rueda, Q/E = zoom | R = reiniciar");
    println!("           SPACE = cambiar modo | P = guardar {RENDER_PATH} | clic = índice | ESC = salir");

    let mut last_mouse: Option<(f32, f32)> = None;
    let mut drag_distance = 0.0;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Rotar arrastrando; un clic sin arrastrar selecciona un triángulo.
        let mouse = window.get_mouse_pos(MouseMode::Discard);
        if window.get_mouse_down(MouseButton::Left) {
            if let Some((mx, my)) = mouse {
                if let Some((lx, ly)) = last_mouse {
                    view.yaw += (mx - lx) * 0.01;
                    view.pitch += (my - ly) * 0.01;
                    drag_distance += (mx - lx).abs() + (my - ly).abs();
                } else {
                    drag_distance = 0.0;
                }
                last_mouse = Some((mx, my));
            }
        } else if let Some((lx, ly)) = last_mouse.take() {
            if drag_distance < 4.0 {
                let id = framebuffer.id_at(lx as usize, ly as usize);
                settings.selected = (id >= 0).then_some(id as usize);
                match settings.selected {
                    Some(i) => describe_triangle(&scene, i),
                    None => println!("Clic en el fondo, ningún triángulo seleccionado"),
                }
            }
        }

        if let Some((_, scroll)) = window.get_scroll_wheel() {
            view.zoom *= 1.1f32.powf(scroll.signum());
        }
        if window.is_key_down(Key::E) {
            view.zoom *= 1.02;
        }
        if window.is_key_down(Key::Q) {
            view.zoom /= 1.02;
        }
        view.zoom = view.zoom.clamp(0.2, 6.0);

        if window.is_key_down(Key::Left) {
            view.yaw -= 0.03;
        }
        if window.is_key_down(Key::Right) {
            view.yaw += 0.03;
        }
        if window.is_key_down(Key::Up) {
            view.pitch -= 0.03;
        }
        if window.is_key_down(Key::Down) {
            view.pitch += 0.03;
        }

        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            view = View::default();
            settings.selected = None;
        }
        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            settings.mode = settings.mode.next();
        }

        framebuffer.clear();
        render(&mut framebuffer, &scene, &view, &settings);

        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            match framebuffer.save_png(RENDER_PATH) {
                Ok(()) => println!("Captura guardada en {RENDER_PATH}"),
                Err(e) => eprintln!("No se pudo guardar {RENDER_PATH}: {e}"),
            }
        }

        let new_title = window_title(&settings);
        if new_title != title {
            window.set_title(&new_title);
            title = new_title;
        }

        window
            .update_with_buffer(&framebuffer.buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .expect("No se pudo actualizar la ventana");
    }
}
