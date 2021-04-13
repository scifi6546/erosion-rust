use super::prelude::{
    insert_terrain, AssetManager, Grid, Model, RenderingContext, RuntimeModel, ShaderBind,
    Transform,
};
use egui::CtxRef;
use legion::World;
use log::{error, info};
use nalgebra::{Vector2, Vector3};
mod pgm_parser;
pub struct TerrainLibrary {
    entries: Vec<Scenario>,
}
impl Default for TerrainLibrary {
    fn default() -> Self {
        Self {
            entries: vec![
                Scenario {
                    name: "Cone World".to_string(),
                    terrain_ctor: Box::new(|| {
                        Terrain::new_cone(
                            Vector2::new(20, 20),
                            Vector2::new(10.0, 10.0),
                            10.0,
                            -1.0,
                        )
                    }),
                },
                Scenario {
                    name: "Small Cone World".to_string(),
                    terrain_ctor: Box::new(|| {
                        Terrain::new_cone(Vector2::new(5, 5), Vector2::new(10.0, 10.0), 10.0, 1.0)
                    }),
                },
                Scenario {
                    name: "Toture Test".to_string(),
                    terrain_ctor: Box::new(|| {
                        Terrain::new_cone(
                            Vector2::new(100, 100),
                            Vector2::new(50.0, 50.0),
                            50.0,
                            -1.0,
                        )
                    }),
                },
                Scenario {
                    name: "PGM File".to_string(),
                    terrain_ctor: Box::new(|| {
                        Terrain::from_pgm(include_bytes!("heightmaps/output.pgm").to_vec(), 0.01)
                            .unwrap()
                    }),
                },
                Scenario {
                    name: "PGM File No Skiiers".to_string(),
                    terrain_ctor: Box::new(|| {
                        Terrain::from_pgm(include_bytes!("heightmaps/output.pgm").to_vec(), 0.01)
                            .unwrap()
                    }),
                },
                Scenario {
                    name: "Volcano".to_string(),
                    terrain_ctor: Box::new(|| {
                        Terrain::from_pgm(include_bytes!("heightmaps/cone.pgm").to_vec(), 0.001)
                            .unwrap()
                    }),
                },
            ],
        }
    }
}
pub struct Scenario {
    pub name: String,
    pub terrain_ctor: Box<dyn Fn() -> Terrain>,
}
impl Scenario {
    pub fn build_scenario(
        &self,
        world: &mut World,
        graphics: &mut RenderingContext,
        asset_manager: &mut AssetManager<RuntimeModel>,
        bound_shader: &ShaderBind,
    ) {
        world.clear();
        info!("building scene: {}", self.name);

        insert_terrain(
            (self.terrain_ctor)(),
            world,
            graphics,
            asset_manager,
            bound_shader.get_bind(),
        )
        .expect("failed to insert terrain");
    }
}
impl TerrainLibrary {
    pub fn draw_gui(
        &self,
        world: &mut World,
        context: &mut CtxRef,
        graphics: &mut RenderingContext,
        asset_manager: &mut AssetManager<RuntimeModel>,
        bound_shader: &ShaderBind,
    ) {
        egui::Window::new("Scenarios").show(context, |ui| {
            for t in self.entries.iter() {
                ui.label(t.name.to_string());
                if ui.button("").clicked {
                    t.build_scenario(world, graphics, asset_manager, bound_shader);
                }
            }
        });
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum TileType {
    Snow,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub height: f32,
    pub tile_type: TileType,
}

pub struct Terrain {
    heights: Grid<f32>,
    velocity: Grid<Vector2<f32>>,
    dimensions: Vector2<usize>,
}

impl Terrain {
    const DELTA_T: f32 = 0.001;
    /// Builds cone terrain with centar at center and slope of `slope`
    pub fn new_cone(
        dimensions: Vector2<usize>,
        center: Vector2<f32>,
        center_height: f32,
        slope: f32,
    ) -> Self {
        let mut heights = vec![];
        heights.reserve(dimensions.x * dimensions.y);
        for x in 0..dimensions.x {
            for y in 0..dimensions.y {
                let radius = ((x as f32 - center.x).powi(2) + (y as f32 - center.y).powi(2)).sqrt();
                let height = center_height + radius * slope;
                heights.push(height);
            }
        }
        Self {
            heights: Grid::from_vec(heights, dimensions),
            velocity: Grid::from_vec(
                vec![Vector2::new(0.0, 0.0); (dimensions.x + 1) * (dimensions.y + 1)],
                Vector2::new(dimensions.x + 1, dimensions.y + 1),
            ),
            dimensions,
        }
    }

    pub fn from_pgm(data: Vec<u8>, scaling: f32) -> Option<Self> {
        if let Ok(s) = String::from_utf8(data) {
            match pgm_parser::terrain_from_pgm(s, TileType::Snow, scaling) {
                Ok(t) => Some(t),
                Err(e) => {
                    error!("{:?}", e);
                    None
                }
            }
        } else {
            None
        }
    }
    fn get_index(&self, x: i32, y: i32) -> usize {
        let x = if x < 0 {
            self.dimensions.x - 1
        } else {
            x as usize % self.dimensions.x
        };
        let y = if y < 0 {
            self.dimensions.y - 1
        } else {
            y as usize % self.dimensions.y
        };
        x * self.dimensions.y + y
    }
    pub fn water_simulation(&mut self) {
        //Update Velocities
        let mut new_velocities = self.velocity.clone();
        for x in 0..self.dimensions.x {
            for y in 0..self.dimensions.y {
                let water_x_n1 = if x > 0 {
                    self.heights
                        .get_unchecked(Vector2::new(x as i64 - 1, y as i64))
                } else {
                    self.heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };
                let water_x_p1 = if x <= self.dimensions.x - 2 {
                    self.heights
                        .get_unchecked(Vector2::new(x as i64 + 1, y as i64))
                } else {
                    self.heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };
                let water_y_n1 = if y > 0 {
                    self.heights
                        .get_unchecked(Vector2::new(x as i64, y as i64 - 1))
                } else {
                    self.heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };
                let water_y_p1 = if y <= self.dimensions.y - 2 {
                    self.heights
                        .get_unchecked(Vector2::new(x as i64, y as i64 + 1))
                } else {
                    self.heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };

                let v = new_velocities.get_mut_unchecked(Vector2::new(x as i64, y as i64));
                let center = self.heights.get_unchecked(Vector2::new(x as i64, y as i64));
                v.x += (water_x_n1 - center) * Self::DELTA_T;
                v.y += (water_y_p1 - center) * Self::DELTA_T;
            }
        }
        self.velocity = new_velocities;
        let mut new_heights = self.heights.clone();
        //Update Water

        for x in 0..self.dimensions.x {
            for y in 0..self.dimensions.y {
                let (water_yn1, v_yn1) = if y > 0 {
                    (
                        self.heights
                            .get_unchecked(Vector2::new(x as i64, y as i64 - 1)),
                        self.velocity
                            .get_unchecked(Vector2::new(x as i64, y as i64 - 1))
                            .y,
                    )
                } else {
                    (
                        self.heights.get_unchecked(Vector2::new(x as i64, y as i64)),
                        self.velocity
                            .get_unchecked(Vector2::new(x as i64, y as i64))
                            .y,
                    )
                };
                let (water_0, v_y0, u_x0) = (
                    self.heights.get_unchecked(Vector2::new(x as i64, y as i64)),
                    self.velocity
                        .get_unchecked(Vector2::new(x as i64, y as i64))
                        .y,
                    self.velocity
                        .get_unchecked(Vector2::new(x as i64, y as i64))
                        .x,
                );
                let (water_y1, v_y1) = if y <= self.dimensions.y - 2 {
                    (
                        self.heights
                            .get_unchecked(Vector2::new(x as i64, y as i64 + 1)),
                        self.velocity
                            .get_unchecked(Vector2::new(x as i64, y as i64 + 1))
                            .y,
                    )
                } else {
                    (
                        self.heights.get_unchecked(Vector2::new(x as i64, y as i64)),
                        self.velocity
                            .get_unchecked(Vector2::new(x as i64, y as i64))
                            .y,
                    )
                };
                let (water_xn1, u_xn1) = if x > 0 {
                    (
                        self.heights
                            .get_unchecked(Vector2::new(x as i64 - 1, y as i64)),
                        self.velocity
                            .get_unchecked(Vector2::new(x as i64 - 1, y as i64))
                            .x,
                    )
                } else {
                    (
                        self.heights.get_unchecked(Vector2::new(x as i64, y as i64)),
                        self.velocity
                            .get_unchecked(Vector2::new(x as i64, y as i64))
                            .x,
                    )
                };
                let (water_x1, u_x1) = if x <= self.dimensions.x - 2 {
                    (
                        self.heights
                            .get_unchecked(Vector2::new(x as i64 + 1, y as i64)),
                        self.velocity
                            .get_unchecked(Vector2::new(x as i64 + 1, y as i64))
                            .x,
                    )
                } else {
                    (
                        self.heights.get_unchecked(Vector2::new(x as i64, y as i64)),
                        self.velocity
                            .get_unchecked(Vector2::new(x as i64, y as i64))
                            .x,
                    )
                };
                let water_xn1_avg = (water_xn1 + water_0) / 2.0;
                let water_x1_avg = (water_x1 + water_0) / 2.0;

                let water_yn1_avg = (water_yn1 + water_0) / 2.0;
                let water_y1_avg = (water_y1 + water_0) / 2.0;
                let deltax = (u_x0 * water_xn1_avg) - (u_x1 * water_x1_avg);
                let deltay = (v_y0 * water_yn1_avg) - (v_y1 * water_y1_avg);
                *new_heights.get_mut_unchecked(Vector2::new(x as i64, y as i64)) +=
                    (deltax + deltay) * Self::DELTA_T;
            }
        }
        self.heights = new_heights;
    }
    pub fn from_tiles(heights: Vec<f32>, dimensions: Vector2<usize>) -> Self {
        Self {
            heights: Grid::from_vec(heights, dimensions),
            velocity: Grid::from_vec(
                vec![Vector2::new(0.0, 0.0); (dimensions.x + 1) * (dimensions.y + 1)],
                dimensions,
            ),
            dimensions,
        }
    }

    pub fn model(&self) -> Model {
        let heights = self.heights.data.iter().copied().collect();
        Model::from_heights(heights, self.dimensions, Transform::default())
    }
    pub fn get_transform_rounded(&self, coordinate: &Vector2<f32>) -> Vector3<f32> {
        let x: i64 = unsafe { coordinate.x.to_int_unchecked() };
        let y: i64 = unsafe { coordinate.y.to_int_unchecked() };
        let convert_dimensions = |i: i64, i_dimensions: i64| {
            if i >= i_dimensions {
                i_dimensions - 1
            } else if i < 0 {
                0
            } else {
                i
            }
        };
        self.get_transform(&Vector2::new(
            convert_dimensions(x, self.dimensions.x as i64),
            convert_dimensions(y, self.dimensions.y as i64),
        ))
        .unwrap()
    }
    pub fn get_transform(&self, coordinate: &Vector2<i64>) -> Option<Vector3<f32>> {
        let pos = coordinate.x as usize * self.dimensions.y + coordinate.y as usize;
        if let Some(height) = self.heights.get(*coordinate) {
            Some(Vector3::new(
                coordinate.x as f32,
                *height,
                coordinate.y as f32,
            ))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
struct HeightVelocityField {
    dimensions: Vector2<usize>,
    velocities: Vec<f32>,
    heights: Vec<f32>,
}
struct VelocitiesOut {
    x_plus: f32,
    x_minus: f32,
    y_plus: f32,
    y_minus: f32,
}

struct Bar {}
impl HeightVelocityField {
    pub fn from_heights(dimensions: Vector2<usize>, heights: Vec<f32>) -> Self {
        assert_eq!(heights.len(), dimensions.x * dimensions.y);
        let velocities = vec![0.0; dimensions.x * dimensions.y - (dimensions.y + 1) / 2];
        Self {
            dimensions,
            velocities,
            heights,
        }
    }
    pub fn get_height(&self, cord: Vector2<usize>) -> f32 {
        self.heights[cord.x * self.dimensions.y + cord.y]
    }
    /// Gets flow going from start.
    /// Sign convention: positive if start is losing material to end
    pub fn get_velocities(&self, cord: Vector2<usize>) -> VelocitiesOut {
        let x_plus = self.velocities[if cord.x < self.dimensions.x - 1 {
            (cord.x * 2 + 1) * self.dimensions.y * 2 + cord.y - self.dimensions.x
        } else {
            todo!()
        }];
        let x_minus = -1.0
            * self.velocities[if cord.x > 0 {
                (cord.x * 2 - 1) * self.dimensions.y * 2 + cord.y - self.dimensions.x
            } else {
                todo!()
            }];
        let y_plus = self.velocities[if cord.y < self.dimensions.y - 1 {
            (cord.x * 2) * self.dimensions.y * 2 + cord.y + 1 - self.dimensions.x
        } else {
            todo!()
        }];
        let y_minus = -1.0
            * self.heights[if cord.y > 0 {
                (cord.x * 2) * self.dimensions.y * 2 + cord.y - 1 - self.dimensions.x
            } else {
                todo!()
            }];
        VelocitiesOut {
            x_plus,
            x_minus,
            y_plus,
            y_minus,
        }
    }
}
