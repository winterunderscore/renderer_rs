use std::fs::File;
use std::io::{prelude::*, BufReader};

use speedy2d::color::Color;

type TYPE = f32;

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: TYPE,
    pub y: TYPE,
    pub z: TYPE,
    pub w: TYPE,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    pub const fn new(x: TYPE, y: TYPE, z: TYPE) -> Self {
        Vec3 { x, y, z, w: 1.0 }
    }

    #[allow(dead_code)]
    pub const fn new_with_w(x: TYPE, y: TYPE, z: TYPE, w: TYPE) -> Self {
        Vec3 { x, y, z, w }
    }

    #[allow(dead_code)]
    pub fn add(self: Vec3, other: Vec3) -> Self {
        Vec3::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z
        )
    }

    #[allow(dead_code)]
    pub fn sub(self: Vec3, other: Vec3) -> Self {
        Vec3::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z
        )
    }

    #[allow(dead_code)]
    pub fn mul(self: Vec3, k: TYPE) -> Self {
        Vec3::new(
            self.x * k,
            self.y * k,
            self.z * k
        )
    }
    #[allow(dead_code)]
    pub fn div(self: Vec3, k: TYPE) -> Self {
        Vec3::new(
            self.x / k,
            self.y / k,
            self.z / k
        )
    }

    #[allow(dead_code)]
    pub fn dot_product(self: Vec3, other: Vec3) -> TYPE {
        self.x*other.x + self.y*other.y + self.z*other.z
    }
    #[allow(dead_code)]
    pub fn length(self: Vec3) -> TYPE {
        self.dot_product(self).sqrt()
    }
    #[allow(dead_code)]
    pub fn normalize(self: Vec3) -> Self {
        self.div(self.length())
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub p: [Vec3; 3],
    pub col: Color,
}


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
            p: [Vec3::ZERO; 3],
            col: Color::WHITE,
        }
    }
}

#[derive(Debug, Default)]
pub struct Mesh {
    pub tris: Vec<Triangle>,
}


impl Mesh {
    pub fn load_from_obj_file(file_path: &str) -> Option<Mesh> {
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
                let mut vector: Vec3 = Vec3::ZERO;
                let mut split = line.split_whitespace();


                split.next();
                vector.x = split.next()?.parse::<TYPE>().unwrap();
                vector.y = split.next()?.parse::<TYPE>().unwrap();
                vector.z = split.next()?.parse::<TYPE>().unwrap();

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

#[derive(Debug, Default, Clone, Copy)]
pub struct Mat4x4 {
    pub m: [[TYPE;4];4]
}

impl Mat4x4 {
    #[allow(dead_code)]
    pub fn multiply_vector(self, i: Vec3) -> Vec3 {
        Vec3::new_with_w(
            i.x * self.m[0][0] + i.y * self.m[1][0] + i.z * self.m[2][0] + i.w * self.m[3][0],
            i.x * self.m[0][1] + i.y * self.m[1][1] + i.z * self.m[2][1] + i.w * self.m[3][1],
            i.x * self.m[0][2] + i.y * self.m[1][2] + i.z * self.m[2][2] + i.w * self.m[3][2],
            i.x * self.m[0][3] + i.y * self.m[1][3] + i.z * self.m[2][3] + i.w * self.m[3][3]
        )
    }
}
