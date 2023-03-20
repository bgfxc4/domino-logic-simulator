pub struct Domino {
    pub position: cgmath::Point3<f32>,
    pub rotation_y: f32,
    pub fall_rotation: f32, // -90-90 deg, around centerline of base area
    pub scale: cgmath::Vector3<f32>, // width, heith, depth
    pub id: u32,
}

pub struct Simulator {
    pub dominos: Vec<Domino>,
}

impl Simulator {
    pub fn new() -> Self {
        Simulator {
            dominos: vec![
                Domino {
                    position: cgmath::Point3{x: 0.0, y: 0.0, z: 0.0},
                    rotation_y: 0.0,
                    fall_rotation: 0.0,
                    scale: cgmath::Vector3{x: 1.0, y: 1.0, z: 1.0},
                    id: 0,
                },
                Domino {
                    position: cgmath::Point3{x: 1.0, y: 0.0, z: 1.0},
                    rotation_y: 0.0,
                    fall_rotation: 90.0,
                    scale: cgmath::Vector3{x: 1.0, y: 1.0, z: 1.0},
                    id: 1,
                },

                Domino {
                    position: cgmath::Point3{x: 0.0, y: 0.0, z: 2.0},
                    rotation_y: 90.0,
                    fall_rotation: 45.0,
                    scale: cgmath::Vector3{x: 1.0, y: 1.0, z: 1.0},
                    id: 2,
                },

                Domino {
                    position: cgmath::Point3{x: 0.3, y: 0.0, z: 2.0},
                    rotation_y: 0.0,
                    fall_rotation: 10.0,
                    scale: cgmath::Vector3{x: 1.0, y: 1.0, z: 1.0},
                    id: 3,
                }
            ]
        }
    }
}
