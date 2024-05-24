use std::time::Instant;

use speedy2d::color::Color;
use speedy2d::{Graphics2D, Window};
use speedy2d::window::{WindowHandler, WindowHelper};
use speedy2d::dimen::{Vector2, Vec2};

mod geometry;
use crate::geometry::{
    Vec3,
    Triangle,
    Mesh,
    Mat4x4
};

const WINDOW_SIZE: (u32,u32) = (512,480);
const DRAW_TRIANGLE: bool = false;
const DRAW_WIREFRAME: bool = true;
const PAINTERS_ALGO: bool = false;
const OBJ_PATH: &str = "src/objects/teapot.obj";

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

        graphics.draw_circle(p1, 2.0, color);
        graphics.draw_circle(p2, 2.0, color);
        graphics.draw_circle(p3, 2.0, color);
    }
}

fn main() {
    let window = Window::new_centered("Title", WINDOW_SIZE).unwrap();
    let mesh = Mesh::load_from_obj_file(OBJ_PATH).expect("failed to open obj");

    // Projection Matrix
    
    let near: f32 = 0.1;
    let far: f32 = 10000.0;
    let fov: f32 = 90.0;
    let aspect_ratio: f32 = (WINDOW_SIZE.0 / WINDOW_SIZE.1) as f32;
    let fov_rad: f32 = 1.0 / (fov * 0.5 / 180.0 * 3.14159).tan();
    
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
        camera: Vec3::ZERO,
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

        let elapsed_time = self.start_time.elapsed().as_secs_f32();

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

            tri_rotated_z.p[0] = mat_rot_z.multiply_vector(tri.p[0]);
            tri_rotated_z.p[1] = mat_rot_z.multiply_vector(tri.p[1]);
            tri_rotated_z.p[2] = mat_rot_z.multiply_vector(tri.p[2]);

            tri_rotated_zx.p[0] = mat_rot_x.multiply_vector(tri_rotated_z.p[0]);
            tri_rotated_zx.p[1] = mat_rot_x.multiply_vector(tri_rotated_z.p[1]);
            tri_rotated_zx.p[2] = mat_rot_x.multiply_vector(tri_rotated_z.p[2]);

            tri_translated = tri_rotated_zx.clone();
            tri_translated.p[0].z = tri_rotated_zx.p[0].z + 5.0;
            tri_translated.p[1].z = tri_rotated_zx.p[1].z + 5.0;
            tri_translated.p[2].z = tri_rotated_zx.p[2].z + 5.0;

            let mut normal: Vec3 = Vec3::ZERO;
            let mut line1: Vec3 = Vec3::ZERO;
            let mut line2: Vec3 = Vec3::ZERO;

            line1.x = tri_translated.p[1].x - tri_translated.p[0].x;
            line1.y = tri_translated.p[1].y - tri_translated.p[0].y;
            line1.z = tri_translated.p[1].z - tri_translated.p[0].z;

            line2.x = tri_translated.p[2].x - tri_translated.p[0].x;
            line2.y = tri_translated.p[2].y - tri_translated.p[0].y;
            line2.z = tri_translated.p[2].z - tri_translated.p[0].z;

            normal.x = line1.y * line2.z - line1.z * line2.y;
            normal.y = line1.z * line2.x - line1.x * line2.z;
            normal.z = line1.x * line2.y - line1.y * line2.x;

            let l: f32 = (normal.x.powi(2) + normal.y.powi(2) + normal.z.powi(2)).sqrt();
            normal.x /= l; normal.y /= l; normal.z /= l;

            if normal.x * (tri_translated.p[0].x - self.camera.x) +
               normal.y * (tri_translated.p[0].y - self.camera.y) +
               normal.z * (tri_translated.p[0].z - self.camera.z) < 0.0
            {
                // Lighting
                let mut light_direction: Vec3 = Vec3::new(0.0,0.0,-1.0);
                let l: f32 = (light_direction.x.powi(2) + light_direction.y.powi(2) + light_direction.z.powi(2)).sqrt();
                light_direction.x /= l; light_direction.y /= l; light_direction.z /= l;

                let dp: f32 = normal.x * light_direction.x + normal.y * light_direction.y + normal.z * light_direction.z; 
                tri_translated.col = Color::from_gray(dp as f32);

                // Projection
                tri_projected.p[0] = self.mat_proj.multiply_vector(tri_translated.p[0]);
                tri_projected.p[1] = self.mat_proj.multiply_vector(tri_translated.p[1]);
                tri_projected.p[2] = self.mat_proj.multiply_vector(tri_translated.p[2]);
                tri_projected.col = tri_translated.col;

                tri_projected.p[0] = tri_projected.p[0].div(tri_projected.p[0].w);
                tri_projected.p[1] = tri_projected.p[1].div(tri_projected.p[1].w);
                tri_projected.p[2] = tri_projected.p[2].div(tri_projected.p[2].w);

                // Scale to view
                
                let screen_size = helper.get_size_pixels();
                tri_projected.p[0].x += 1.0; tri_projected.p[0].y += 1.0;
                tri_projected.p[1].x += 1.0; tri_projected.p[1].y += 1.0;
                tri_projected.p[2].x += 1.0; tri_projected.p[2].y += 1.0;
                
                tri_projected.p[0].x *= 0.5 * (screen_size.x as f32);
                tri_projected.p[1].x *= 0.5 * (screen_size.x as f32);
                tri_projected.p[2].x *= 0.5 * (screen_size.x as f32);

                tri_projected.p[0].y *= 0.5 * (screen_size.y as f32);
                tri_projected.p[1].y *= 0.5 * (screen_size.y as f32);
                tri_projected.p[2].y *= 0.5 * (screen_size.y as f32);

                if PAINTERS_ALGO {
                    tris_to_raster.push(tri_projected);
                } else {
                    draw_triangle(graphics,
                        tri_projected.p[0],
                        tri_projected.p[1],
                        tri_projected.p[2],
                        tri_projected.col);
                }
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
