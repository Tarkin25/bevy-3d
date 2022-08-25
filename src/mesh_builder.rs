use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use super::vec3;

#[derive(Debug, Clone, Copy)]
pub struct MeshBuilderSettings {
    pub voxel_size: f32,
}

impl Default for MeshBuilderSettings {
    fn default() -> Self {
        Self { voxel_size: 1.0 }
    }
}

#[derive(Debug, Default)]
pub struct MeshBuilder {
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    vertex_count: u32,
    normals: Vec<Vec3>,
    position: Vec3,
    settings: MeshBuilderSettings,
}

impl MeshBuilder {
    pub fn new(settings: MeshBuilderSettings) -> Self {
        Self {
            settings,
            vertices: Default::default(),
            indices: Default::default(),
            vertex_count: Default::default(),
            normals: Default::default(),
            position: Default::default(),
        }
    }
    
    pub fn build(self) -> Mesh {
        let vertices: Vec<_> = self
            .vertices
            .into_iter()
            .map(|Vec3 { x, y, z }| [x, y, z])
            .collect();
        let normals: Vec<_> = self
            .normals
            .into_iter()
            .map(|Vec3 { x, y, z }| [x, y, z])
            .collect();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_indices(Some(Indices::U32(self.indices)));

        mesh
    }

    pub fn move_to(&mut self, position: Vec3) {
        self.position = position;
    }

    fn add_face(&mut self, unit_vertices: [Vec3; 4], normal: Vec3) {
        self.vertices.extend(unit_vertices.map(|v| v * self.settings.voxel_size + self.position));
        self.indices.extend([0, 1, 2, 2, 3, 0].map(|i| i + self.vertex_count));
        self.normals.extend([normal; 4]);
        self.vertex_count += 4;
    }

    pub fn face_top(&mut self) {
        self.add_face([
            vec3!(0, 1, 0),
            vec3!(0, 1, 1),
            vec3!(1, 1, 1),
            vec3!(1, 1, 0),
        ], Vec3::Y);
    }

    pub fn face_bottom(&mut self) {
        self.add_face([
            vec3!(0, 0, 0),
            vec3!(1, 0, 0),
            vec3!(1, 0, 1),
            vec3!(0, 0, 1),
        ], Vec3::NEG_Y);
    }

    pub fn face_front(&mut self) {
        self.add_face([
            vec3!(0, 0, 0),
            vec3!(0, 1, 0),
            vec3!(1, 1, 0),
            vec3!(1, 0, 0),
        ], Vec3::NEG_Z);
    }

    pub fn face_back(&mut self) {
        self.add_face([
            vec3!(0, 0, 1),
            vec3!(1, 0, 1),
            vec3!(1, 1, 1),
            vec3!(0, 1, 1),
        ], Vec3::Z);
    }

    pub fn face_right(&mut self) {
        self.add_face([
            vec3!(0, 0, 0),
            vec3!(0, 0, 1),
            vec3!(0, 1, 1),
            vec3!(0, 1, 0),
        ], Vec3::NEG_X);
    }

    pub fn face_left(&mut self) {
        self.add_face([
            vec3!(1, 0, 0),
            vec3!(1, 1, 0),
            vec3!(1, 1, 1),
            vec3!(1, 0, 1),
        ], Vec3::X);
    }
}
