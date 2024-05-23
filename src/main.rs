use std::time::Instant;
use std::fs::File;
use std::io::{prelude::*, BufReader};

use speedy2d::color::Color;
use speedy2d::{Graphics2D, Window};
use speedy2d::window::{WindowHandler, WindowHelper};
use speedy2d::dimen::{Vector2, Vec2};

const WINDOW_SIZE: (u32,u32) = (512,480);
const DRAW_TRIANGLE: bool = true;
const DRAW_WIREFRAME: bool = false;
const OBJ_PATH: &str = "src/objects/teapot.obj";

#[derive(Debug, Default, Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug, Clone)]
struct Triangle {
    p: [Vec3; 3],
    col: Color,
}

#[derive(Debug, Default)]
struct Mesh {
    tris: Vec<Triangle>,
}

#[derive(Debug, Default)]
struct Mat4x4 {
    m: [[f64;4];4]
}

// impl Vec3 {
//     const fn new(x: f64, y: f64, z: f64) {
//         Vec3 { x, y, z, 0.0 }
//     }
// }

impl Triangle {
    fn new(p: [Vec3; 3]) -> Triangle {
        Triangle {
            p: p,
            .. Default::default()
        }
    }
}

impl Default for Triangle {
    fn default() -> Triangle {
        Triangle {
            p: [Default::default(); 3],
            col: Color::WHITE,
        }
    }
}

impl Mesh {
    fn load_from_obj_file(file_path: &str) -> Option<Mesh> {
        let file = File::open(file_path).ok()?;
        let reader = BufReader::new(file);

        let mut tris: Vec<Triangle> = vec![];
        let mut verts: Vec<Vec3> = vec![];

        for line in reader.lines() {
            let line = line.ok()?;
            let mut chars = line.chars();
            let ch = match chars.next() {
                Some(ch) => ch,
                None => ' ',
            };

            if chars.next() != Some(' ') { continue };
            if ch == 'v' {
                let mut vector: Vec3 = Default::default();
                let mut split = line.split_whitespace();


                split.next();
                vector.x = split.next()?.parse::<f64>().unwrap();
                vector.y = split.next()?.parse::<f64>().unwrap();
                vector.z = split.next()?.parse::<f64>().unwrap();

                verts.push(vector);
            }

            if ch == 'f' {
                let mut split = line.split_whitespace();

                split.next();
                let v1 = split.next()?.parse::<usize>().unwrap();
                let v2 = split.next()?.parse::<usize>().unwrap();
                let v3 = split.next()?.parse::<usize>().unwrap();

                tris.push(Triangle::new([
                    verts[v1 - 1],
                    verts[v2 - 1],
                    verts[v3 - 1],
                ]));
            }
        }

        Some(Mesh {
            tris: tris,
        })
    }
}

fn multiply_matrix_vector(i: Vec3, o: &mut Vec3, m: &Mat4x4) {
    o.x = i.x * m.m[0][0] + i.y * m.m[1][0] + i.z * m.m[2][0] + m.m[3][0];
    o.y = i.x * m.m[0][1] + i.y * m.m[1][1] + i.z * m.m[2][1] + m.m[3][1];
    o.z = i.x * m.m[0][2] + i.y * m.m[1][2] + i.z * m.m[2][2] + m.m[3][2];
    
    let w: f64 = i.x * m.m[0][3] + i.y * m.m[1][3] + i.z * m.m[2][3] + m.m[3][3];
    if w != 0.0 {
        o.x /= w;
        o.y /= w;
        o.z /= w;
    }
}

fn draw_triangle(graphics: &mut Graphics2D, v1: Vec3, v2: Vec3, v3: Vec3, color: Color) {
    let p1: Vec2 = Vector2::new(v1.x, v1.y).into_f32();
    let p2: Vec2 = Vector2::new(v2.x, v2.y).into_f32(); 
    let p3: Vec2 = Vector2::new(v3.x, v3.y).into_f32(); 

    if DRAW_TRIANGLE {
        graphics.draw_triangle([p1, p2, p3], color);
    }

    if DRAW_WIREFRAME {
        graphics.draw_line(p1, p2, 1.0, Color::RED);
        graphics.draw_line(p2, p3, 1.0, Color::GREEN);
        graphics.draw_line(p3, p1, 1.0, Color::BLUE);

        graphics.draw_circle(p1, 3.0, color);
        graphics.draw_circle(p2, 3.0, color);
        graphics.draw_circle(p3, 3.0, color);
    }
}

fn main() {
    let window = Window::new_centered("Title", WINDOW_SIZE).unwrap();
    let mesh = Mesh::load_from_obj_file(OBJ_PATH).expect("failed to open obj");

    // Projection Matrix
    
    let near: f64 = 0.1;
    let far: f64 = 10000.0;
    let fov: f64 = 90.0;
    let aspect_ratio: f64 = (WINDOW_SIZE.0 / WINDOW_SIZE.1).into();
    let fov_rad: f64 = 1.0 / (fov * 0.5 / 180.0 * 3.14159).tan();
    
    let mut mat_proj: Mat4x4 = Default::default();

    mat_proj.m[0][0] = aspect_ratio * fov_rad;
    mat_proj.m[1][1] = fov_rad;
    mat_proj.m[2][2] = far / (far - near);
    mat_proj.m[3][2] = (-far * near) / (far - near);
    mat_proj.m[2][3] = 1.0;
    mat_proj.m[3][3] = 0.0;

    window.run_loop(MyWindowHandler{
        start_time: Instant::now(),
        mesh: mesh,
        mat_proj: mat_proj,
        camera: Default::default(),
    });
}

struct MyWindowHandler {
    start_time: Instant,
    mesh: Mesh,
    mat_proj: Mat4x4,
    camera: Vec3,
}

impl WindowHandler for MyWindowHandler
{
    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D)
    {
        graphics.clear_screen(Color::BLACK);

        let elapsed_time = self.start_time.elapsed().as_secs_f64();

        let mut mat_rot_z: Mat4x4 = Default::default();
        let mut mat_rot_x: Mat4x4 = Default::default();
        let theta = 1.0 * elapsed_time;

        mat_rot_z.m[0][0] = theta.cos();
        mat_rot_z.m[0][1] = theta.sin();
        mat_rot_z.m[1][0] = -theta.sin();
        mat_rot_z.m[1][1] = theta.cos();
        mat_rot_z.m[2][2] = 1.0;
        mat_rot_z.m[3][3] = 1.0;

        mat_rot_x.m[0][0] = 1.0;
        mat_rot_x.m[1][1] = (theta * 0.5).cos();
        mat_rot_x.m[1][2] = (theta * 0.5).sin();
        mat_rot_x.m[2][1] = -(theta * 0.5).sin();
        mat_rot_x.m[2][2] = (theta * 0.5).cos();
        mat_rot_x.m[3][3] = 1.0;

        let mut tris_to_raster: Vec<Triangle> = vec![];
        for tri in &self.mesh.tris {
            let mut tri_rotated_z: Triangle = Default::default();
            let mut tri_rotated_zx: Triangle = Default::default();

            let mut tri_projected: Triangle = Default::default();
            let mut tri_translated: Triangle;

            multiply_matrix_vector(tri.p[0], &mut tri_rotated_z.p[0], &mat_rot_z);
            multiply_matrix_vector(tri.p[1], &mut tri_rotated_z.p[1], &mat_rot_z);
            multiply_matrix_vector(tri.p[2], &mut tri_rotated_z.p[2], &mat_rot_z);

            multiply_matrix_vector(tri_rotated_z.p[0], &mut tri_rotated_zx.p[0], &mat_rot_x);
            multiply_matrix_vector(tri_rotated_z.p[1], &mut tri_rotated_zx.p[1], &mat_rot_x);
            multiply_matrix_vector(tri_rotated_z.p[2], &mut tri_rotated_zx.p[2], &mat_rot_x);

            tri_translated = tri_rotated_zx.clone();
            tri_translated.p[0].z = tri_rotated_zx.p[0].z + 5.0;
            tri_translated.p[1].z = tri_rotated_zx.p[1].z + 5.0;
            tri_translated.p[2].z = tri_rotated_zx.p[2].z + 5.0;

            let mut normal: Vec3 = Default::default();
            let mut line1: Vec3 = Default::default();
            let mut line2: Vec3 = Default::default();

            line1.x = tri_translated.p[1].x - tri_translated.p[0].x;
            line1.y = tri_translated.p[1].y - tri_translated.p[0].y;
            line1.z = tri_translated.p[1].z - tri_translated.p[0].z;

            line2.x = tri_translated.p[2].x - tri_translated.p[0].x;
            line2.y = tri_translated.p[2].y - tri_translated.p[0].y;
            line2.z = tri_translated.p[2].z - tri_translated.p[0].z;

            normal.x = line1.y * line2.z - line1.z * line2.y;
            normal.y = line1.z * line2.x - line1.x * line2.z;
            normal.z = line1.x * line2.y - line1.y * line2.x;

            let l: f64 = (normal.x.powi(2) + normal.y.powi(2) + normal.z.powi(2)).sqrt();
            normal.x /= l; normal.y /= l; normal.z /= l;

            if normal.x * (tri_translated.p[0].x - self.camera.x) +
               normal.y * (tri_translated.p[0].y - self.camera.y) +
               normal.z * (tri_translated.p[0].z - self.camera.z) < 0.0
            {
                // Lighting
                let mut light_direction: Vec3 = Vec3 { x: 0.0, y: 0.0, z: -1.0 };
                let l: f64 = (light_direction.x.powi(2) + light_direction.y.powi(2) + light_direction.z.powi(2)).sqrt();
                light_direction.x /= l; light_direction.y /= l; light_direction.z /= l;

                let dp: f64 = normal.x * light_direction.x + normal.y * light_direction.y + normal.z * light_direction.z; 
                tri_translated.col = Color::from_gray(dp as f32);

                // Projection
                multiply_matrix_vector(tri_translated.p[0], &mut tri_projected.p[0], &self.mat_proj);
                multiply_matrix_vector(tri_translated.p[1], &mut tri_projected.p[1], &self.mat_proj);
                multiply_matrix_vector(tri_translated.p[2], &mut tri_projected.p[2], &self.mat_proj);
                tri_projected.col = tri_translated.col;

                // Scale to view
                
                let screen_size = helper.get_size_pixels();
                tri_projected.p[0].x += 1.0; tri_projected.p[0].y += 1.0;
                tri_projected.p[1].x += 1.0; tri_projected.p[1].y += 1.0;
                tri_projected.p[2].x += 1.0; tri_projected.p[2].y += 1.0;
                
                tri_projected.p[0].x *= 0.5 * (screen_size.x as f64);
                tri_projected.p[1].x *= 0.5 * (screen_size.x as f64);
                tri_projected.p[2].x *= 0.5 * (screen_size.x as f64);

                tri_projected.p[0].y *= 0.5 * (screen_size.y as f64);
                tri_projected.p[1].y *= 0.5 * (screen_size.y as f64);
                tri_projected.p[2].y *= 0.5 * (screen_size.y as f64);

                tris_to_raster.push(tri_projected);
            }

            tris_to_raster.sort_unstable_by(|a, b| ((b.p[0].z + b.p[1].z + b.p[2].z) / 3.0).partial_cmp(&((a.p[0].z + a.p[1].z + a.p[2].z) / 3.0)).unwrap());

            for tri in &tris_to_raster {
                draw_triangle(graphics,
                    tri.p[0],
                    tri.p[1],
                    tri.p[2],
                    tri.col);
            }
        }

        helper.request_redraw();
    }
}
