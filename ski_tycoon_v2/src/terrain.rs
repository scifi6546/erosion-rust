use super::prelude::{
    insert_terrain, AssetManager, DeltaCamera, Grid, Model, RenderingContext, RuntimeModel,
    ShaderBind, Transform,
};
use egui::CtxRef;

use legion::*;
use log::{error, info};
use nalgebra::{Vector2, Vector3};
mod pgm_parser;
pub struct TerrainLibrary {
    pub entries: Vec<Scenario>,
}
impl Default for TerrainLibrary {
    fn default() -> Self {
        Self {
            entries: vec![
                Scenario {
                    name: "Droplet".to_string(),
                    terrain_ctor: Box::new(|| {
                        Terrain::droplet(
                            Vector2::new(20, 20),
                            1.0,
                            vec![Droplet {
                                position: Vector2::new(10, 10),
                                height: 6.0,
                            }],
                        )
                    }),
                },
                Scenario {
                    name: "Big Droplet".to_string(),
                    terrain_ctor: Box::new(|| {
                        Terrain::cone_flat(
                            Vector2::new(100, 100),
                            Vector2::new(50.0, 50.0),
                            5.0,
                            -0.5,
                            0.0,
                        )
                    }),
                },
                Scenario {
                    name: "Many Droplets".to_string(),
                    terrain_ctor: Box::new(|| {
                        Terrain::droplet(
                            Vector2::new(50, 50),
                            1.0,
                            vec![
                                Droplet {
                                    position: Vector2::new(10, 10),
                                    height: 6.0,
                                },
                                Droplet {
                                    position: Vector2::new(20, 20),
                                    height: 6.0,
                                },
                                Droplet {
                                    position: Vector2::new(0, 10),
                                    height: 6.0,
                                },
                                Droplet {
                                    position: Vector2::new(23, 28),
                                    height: 6.0,
                                },
                                Droplet {
                                    position: Vector2::new(43, 38),
                                    height: 6.0,
                                },
                                Droplet {
                                    position: Vector2::new(33, 8),
                                    height: 6.0,
                                },
                                Droplet {
                                    position: Vector2::new(2, 44),
                                    height: 6.0,
                                },
                            ],
                        )
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
        camera: &mut DeltaCamera,
        graphics: &mut RenderingContext,
        asset_manager: &mut AssetManager<RuntimeModel>,
        bound_shader: &ShaderBind,
    ) {
        world.clear();
        info!("building scene: {}", self.name);
        let terrain = (self.terrain_ctor)();

        camera.set_translation(Vector3::new(
            terrain.dimensions.x as f32 / 2.0,
            0.0,
            terrain.dimensions.y as f32 / 2.0,
        ));
        camera.set_radius((terrain.dimensions.x as f32 * terrain.dimensions.y as f32).sqrt());
        insert_terrain(
            terrain,
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
        camera: &mut DeltaCamera,
        context: &mut CtxRef,
        graphics: &mut RenderingContext,
        asset_manager: &mut AssetManager<RuntimeModel>,
        bound_shader: &ShaderBind,
    ) {
        egui::Window::new("How do I use this?").show(context, |ui| {

            ui.label("Click on a scenario to start simulation\nScroll to zoom\n Left click+drag to change view");
        });
        egui::Window::new("Scenarios").show(context, |ui| {
            for t in self.entries.iter() {
                ui.label(t.name.to_string());
                if ui.button("Click Here").clicked {
                    t.build_scenario(world, camera, graphics, asset_manager, bound_shader);
                }
            }
        });
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub height: f32,
}

pub struct Terrain {
    heights: Grid<f32>,
    velocity: Grid<Vector2<f32>>,
    dimensions: Vector2<usize>,
}
pub struct Droplet {
    position: Vector2<usize>,
    height: f32,
}

impl Terrain {
    const DELTA_T: f32 = 0.01;
    const G: f32 = 0.1;
    const VISC: f32 = 0.0002;
    /// Builds cone terrain with center at center and slope of `slope`
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
    pub fn flat(dimensions: Vector2<usize>, height: f32) -> Self {
        Self {
            heights: Grid::from_vec(vec![height; dimensions.x * dimensions.y], dimensions),
            velocity: Grid::from_vec(
                vec![Vector2::new(0.0, 0.0); (dimensions.x + 1) * (dimensions.y + 1)],
                Vector2::new(dimensions.x + 1, dimensions.y + 1),
            ),
            dimensions,
        }
    }
    pub fn cone_flat(
        dimensions: Vector2<usize>,
        center: Vector2<f32>,
        center_height: f32,
        slope: f32,
        min_height: f32,
    ) -> Self {
        let mut heights = vec![];
        heights.reserve(dimensions.x * dimensions.y);
        for x in 0..dimensions.x {
            for y in 0..dimensions.y {
                let radius = ((x as f32 - center.x).powi(2) + (y as f32 - center.y).powi(2)).sqrt();
                let mut height = center_height + radius * slope;
                if height < min_height {
                    height = min_height;
                }
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
    pub fn droplet(dimensions: Vector2<usize>, height: f32, droplet: Vec<Droplet>) -> Self {
        let mut heights = vec![height; dimensions.x * dimensions.y];
        for drop in droplet.iter() {
            heights[drop.position.x * dimensions.y + drop.position.y] = drop.height;
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
            match pgm_parser::terrain_from_pgm(s, scaling) {
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
    fn update_velocity(
        heights: &Grid<f32>,
        velocity: &Grid<Vector2<f32>>,
        velocity_apply: &Grid<Vector2<f32>>,
        dimensions: &Vector2<usize>,
        delta_t: f32,
    ) -> Grid<Vector2<f32>> {
        let mut new_velocities = velocity_apply.clone();
        //Update Velocities
        for x in 0..dimensions.x {
            for y in 0..dimensions.y {
                let water_x_n1 = if x > 0 {
                    heights.get_unchecked(Vector2::new(x as i64 - 1, y as i64))
                } else {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };
                let water_y_n1 = if y > 0 {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64 - 1))
                } else {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };

                let v = new_velocities.get_mut_unchecked(Vector2::new(x as i64, y as i64));
                let center = heights.get_unchecked(Vector2::new(x as i64, y as i64));
                v.x += (water_x_n1 - center) * delta_t * Self::G;
                v.x -= v.x * Self::VISC;
                if x == 0 {
                    v.x = 0.0;
                }
                v.y += (water_y_n1 - center) * delta_t * Self::G;
                v.y -= v.y * Self::VISC;
                if y == 0 {
                    v.y = 0.0;
                }
            }
        }
        return new_velocities;
    }
    fn update_water(
        heights: &Grid<f32>,
        velocity: &Grid<Vector2<f32>>,
        heights_apply: &Grid<f32>,
        dimensions: &Vector2<usize>,
        delta_t: f32,
    ) -> Grid<f32> {
        let mut heights_out = heights_apply.clone();
        for x in 0..dimensions.x {
            for y in 0..dimensions.y {
                let water_yn1 = if y > 0 {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64 - 1))
                } else {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };
                let (water_0, v_y0, u_x0) = (
                    heights.get_unchecked(Vector2::new(x as i64, y as i64)),
                    velocity.get_unchecked(Vector2::new(x as i64, y as i64)).y,
                    velocity.get_unchecked(Vector2::new(x as i64, y as i64)).x,
                );
                let (water_y1, v_y1) = if y <= dimensions.y - 2 {
                    (
                        heights.get_unchecked(Vector2::new(x as i64, y as i64 + 1)),
                        velocity
                            .get_unchecked(Vector2::new(x as i64, y as i64 + 1))
                            .y,
                    )
                } else {
                    (heights.get_unchecked(Vector2::new(x as i64, y as i64)), 0.0)
                };
                let water_xn1 = if x > 0 {
                    heights.get_unchecked(Vector2::new(x as i64 - 1, y as i64))
                } else {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };
                let (water_x1, u_x1) = if x <= dimensions.x - 2 {
                    (
                        heights.get_unchecked(Vector2::new(x as i64 + 1, y as i64)),
                        velocity
                            .get_unchecked(Vector2::new(x as i64 + 1, y as i64))
                            .x,
                    )
                } else {
                    (heights.get_unchecked(Vector2::new(x as i64, y as i64)), 0.0)
                };
                let water_xn1_avg = (water_xn1 + water_0) / 2.0;
                let water_x1_avg = (water_x1 + water_0) / 2.0;

                let water_yn1_avg = (water_yn1 + water_0) / 2.0;
                let water_y1_avg = (water_y1 + water_0) / 2.0;
                let deltax = (u_x1 * water_x1_avg) - (u_x0 * water_xn1_avg);
                let deltay = (v_y1 * water_y1_avg) - (v_y0 * water_yn1_avg);
                *heights_out.get_mut_unchecked(Vector2::new(x as i64, y as i64)) +=
                    -1.0 * (deltax + deltay) * delta_t;
            }
        }
        return heights_out;
    }
    fn run_timestep(
        heights: &Grid<f32>,
        velocity: &Grid<Vector2<f32>>,
        heights_apply: &mut Grid<f32>,
        velocities_apply: &mut Grid<Vector2<f32>>,
        dimensions: &Vector2<usize>,
        delta_t: f32,
    ) {
        //Update Velocities
        for x in 0..dimensions.x {
            for y in 0..dimensions.y {
                let water_x_n1 = if x > 0 {
                    heights.get_unchecked(Vector2::new(x as i64 - 1, y as i64))
                } else {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };
                let water_y_n1 = if y > 0 {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64 - 1))
                } else {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };

                let v = velocities_apply.get_mut_unchecked(Vector2::new(x as i64, y as i64));
                let center = heights.get_unchecked(Vector2::new(x as i64, y as i64));
                v.x += (water_x_n1 - center) * delta_t * Self::G;
                v.y += (water_y_n1 - center) * delta_t * Self::G;
            }
        }

        //Update Water

        for x in 0..dimensions.x {
            for y in 0..dimensions.y {
                let water_yn1 = if y > 0 {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64 - 1))
                } else {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };
                let (water_0, v_y0, u_x0) = (
                    heights.get_unchecked(Vector2::new(x as i64, y as i64)),
                    velocities_apply
                        .get_unchecked(Vector2::new(x as i64, y as i64))
                        .y,
                    velocities_apply
                        .get_unchecked(Vector2::new(x as i64, y as i64))
                        .x,
                );
                let (water_y1, v_y1) = if y <= dimensions.y - 2 {
                    (
                        heights.get_unchecked(Vector2::new(x as i64, y as i64 + 1)),
                        velocities_apply
                            .get_unchecked(Vector2::new(x as i64, y as i64 + 1))
                            .y,
                    )
                } else {
                    (
                        heights.get_unchecked(Vector2::new(x as i64, y as i64)),
                        velocities_apply
                            .get_unchecked(Vector2::new(x as i64, y as i64))
                            .y,
                    )
                };
                let water_xn1 = if x > 0 {
                    heights.get_unchecked(Vector2::new(x as i64 - 1, y as i64))
                } else {
                    heights.get_unchecked(Vector2::new(x as i64, y as i64))
                };
                let (water_x1, u_x1) = if x <= dimensions.x - 2 {
                    (
                        heights.get_unchecked(Vector2::new(x as i64 + 1, y as i64)),
                        velocities_apply
                            .get_unchecked(Vector2::new(x as i64 + 1, y as i64))
                            .x,
                    )
                } else {
                    (
                        heights.get_unchecked(Vector2::new(x as i64, y as i64)),
                        velocities_apply
                            .get_unchecked(Vector2::new(x as i64, y as i64))
                            .x,
                    )
                };
                let water_xn1_avg = (water_xn1 + water_0) / 2.0;
                let water_x1_avg = (water_x1 + water_0) / 2.0;

                let water_yn1_avg = (water_yn1 + water_0) / 2.0;
                let water_y1_avg = (water_y1 + water_0) / 2.0;
                let deltax = (u_x1 * water_x1_avg) - (u_x0 * water_xn1_avg);
                let deltay = (v_y1 * water_y1_avg) - (v_y0 * water_yn1_avg);
                *heights_apply.get_mut_unchecked(Vector2::new(x as i64, y as i64)) +=
                    -1.0 * (deltax + deltay) * delta_t;
            }
        }
    }
    pub fn water_simulation(&mut self) {
        //Update Velocities
        for _ in 0..20 {
            let half_uv = Self::update_velocity(
                &self.heights,
                &self.velocity,
                &self.velocity,
                &self.dimensions,
                Self::DELTA_T,
            );
            let half_h = Self::update_water(
                &self.heights,
                &self.velocity,
                &self.heights,
                &self.dimensions,
                Self::DELTA_T,
            );

            self.velocity = Self::update_velocity(
                &half_h,
                &half_uv,
                &self.velocity,
                &self.dimensions,
                Self::DELTA_T,
            );
            self.heights = Self::update_water(
                &half_h,
                &half_uv,
                &self.heights,
                &self.dimensions,
                Self::DELTA_T,
            );
        }
    }
    pub fn from_tiles(heights: Vec<f32>, dimensions: Vector2<usize>) -> Self {
        Self {
            heights: Grid::from_vec(heights, dimensions),
            velocity: Grid::from_vec(
                vec![Vector2::new(0.0, 0.0); (dimensions.x + 1) * (dimensions.y + 1)],
                Vector2::new(dimensions.x + 1, dimensions.y + 1),
            ),
            dimensions,
        }
    }

    pub fn model(&self) -> Model {
        Model::from_heights(&self.heights.data, self.dimensions, Transform::default())
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
