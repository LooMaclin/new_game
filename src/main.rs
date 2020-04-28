use macroquad::*;
use std::time::Instant;
use nalgebra as na;
use ncollide2d::shape::{Cuboid, ShapeHandle};
use nphysics2d::world::{DefaultMechanicalWorld, DefaultGeometricalWorld};
use nphysics2d::object::{DefaultBodySet, DefaultColliderSet, RigidBodyDesc, ColliderDesc, BodyPartHandle, DefaultBodyHandle, DefaultColliderHandle, Body, RigidBody};
use nphysics2d::joint::DefaultJointConstraintSet;
use nphysics2d::force_generator::DefaultForceGeneratorSet;
use nphysics2d::math::{Force, ForceType};

async fn load_idle_animation() -> Vec<Texture2D> {
    vec![load_texture("assets/adventurer-idle-2-00.png").await,
         load_texture("assets/adventurer-idle-2-01.png").await,
         load_texture("assets/adventurer-idle-2-02.png").await,
         load_texture("assets/adventurer-idle-2-03.png").await,
    ]
}

async fn load_run_animation() -> Vec<Texture2D> {
    vec![load_texture("assets/adventurer-run-01.png").await,
         load_texture("assets/adventurer-run-02.png").await,
         load_texture("assets/adventurer-run-03.png").await,
         load_texture("assets/adventurer-run-04.png").await,
         load_texture("assets/adventurer-run-05.png").await
    ]
}

async fn load_attack1_animation() -> Vec<Texture2D> {
    vec![load_texture("assets/adventurer-attack1-00.png").await,
         load_texture("assets/adventurer-attack1-01.png").await,
         load_texture("assets/adventurer-attack1-02.png").await,
         load_texture("assets/adventurer-attack1-03.png").await,
         load_texture("assets/adventurer-attack1-04.png").await
    ]
}

struct GameObject {
    body_handle: DefaultBodyHandle,
    collider_handle: DefaultColliderHandle,
    width: f32,
    height: f32,
}

impl GameObject {

    pub fn new(x: f32, y: f32, bodies: &mut DefaultBodySet<f32>, colliders: &mut DefaultColliderSet<f32>, width: f32, height: f32, mass: f32, density: f32) -> Self {
        let translation = na::Vector2::new(x, y);
        let rigid_body_desc = RigidBodyDesc::new().translation(translation).mass(mass).build();
        let body_handle = bodies.insert(rigid_body_desc);
        let cuboid = Cuboid::new(na::Vector2::new(width, height));
        let shape_handle = ShapeHandle::new(cuboid);
        let body_part_handle = BodyPartHandle(body_handle, 0);
        let collider_desc = ColliderDesc::new(shape_handle).density(density).translation(-na::Vector2::y()).build(body_part_handle);
        let collider_handle = colliders.insert(collider_desc);
        Self {
            body_handle,
            collider_handle,
            width,
            height,
        }
    }

    pub fn debug_draw(&self, bodies: &DefaultBodySet<f32>) {
        let pos = bodies.rigid_body(self.body_handle).unwrap().position().translation.vector;
        draw_rectangle(pos.x-self.width, pos.y-self.height, self.width, self.height, RED);
    }

    pub fn rigid_body<'a>(&self, bodies: &'a DefaultBodySet<f32>) -> &'a RigidBody<f32> {
        bodies.rigid_body(self.body_handle).unwrap()
    }

    pub fn rigid_body_mut<'a>(&self, bodies: &'a mut DefaultBodySet<f32>) -> &'a mut dyn Body<f32> {
        bodies.get_mut(self.body_handle).unwrap()
    }
}


#[macroquad::main("Game")]
async fn main() {
    let idle_animation = load_idle_animation().await;
    let run_animation = load_run_animation().await;
    let attack_1_animation = load_attack1_animation().await;
    let animations = vec![idle_animation, run_animation, attack_1_animation];
    let mut current_frame = 0;
    let mut timeline = Instant::now();
    let step = 200.0;
    let mut current_animation = 0;
    let mut flip = false;
    let mut mechanical_world = DefaultMechanicalWorld::new(na::Vector2::new(0.0, 9.81));
    let mut geometrical_world = DefaultGeometricalWorld::new();
    let mut joint_constraints = DefaultJointConstraintSet::new();
    let mut force_generators = DefaultForceGeneratorSet::new();
    let mut bodies = DefaultBodySet::new();
    let mut colliders = DefaultColliderSet::new();

    let ground = GameObject::new(screen_width()/2., 480., &mut bodies, &mut colliders, screen_width(), 10., 0., 0.);
    let block = GameObject::new(100., 400., &mut bodies, &mut colliders, 10., 10., 75., 1.);
    let hero = GameObject::new(10., 350., &mut bodies, &mut colliders, 10., 10., 75., 1.);
    mechanical_world.maintain(&mut geometrical_world,
                              &mut bodies,
                              &mut colliders,
                              &mut joint_constraints,);
    loop {
        mechanical_world.step(
            &mut geometrical_world,
            &mut bodies,
            &mut colliders,
            &mut joint_constraints,
            &mut force_generators,
        );
        clear_background(WHITE);
        ground.debug_draw(&bodies);
        block.debug_draw(&bodies);
        hero.debug_draw(&bodies);
        let elapsed = timeline.elapsed();
        let new_frame = elapsed.as_millis() as f64 / step;
        if new_frame > (animations[current_animation].len() - 1) as f64 {
            timeline = Instant::now();
            current_frame = 0;
        } else {
            current_frame = new_frame as usize;
        }
        let texture = animations[current_animation][current_frame];
        let pos = hero.rigid_body(&bodies).position().translation.vector;
        draw_texture_ex(
            texture,
            pos.x-texture.width()/2.,
            pos.y-texture.height()/2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(texture.width(), texture.height())),
                source: if flip { Some(Rect {
                    x: texture.width(),
                    y: 0.,
                    w: -texture.width(),
                    h: texture.height(),
                }) } else { None },
                rotation: 0.,
            },
        );
        if is_key_down(KeyCode::Right) {
            let force = Force::linear(na::Vector2::new(250., 0.));
            hero.rigid_body_mut(&mut bodies).apply_force(0, &force, ForceType::AccelerationChange, false);
            current_animation = 1;
            flip = false;
        } else if is_key_down(KeyCode::Left) {
            let force = Force::linear(na::Vector2::new(-250., 0.));
            hero.rigid_body_mut(&mut bodies).apply_force(0, &force, ForceType::AccelerationChange, false);
            flip = true;
            current_animation = 1;
        } else {
            current_animation = 0;
        }
        if is_key_down(KeyCode::Space) {
            let force = Force::linear(na::Vector2::new(0., -250.));
            hero.rigid_body_mut(&mut bodies).apply_force(0, &force, ForceType::AccelerationChange, false);
        }

        if is_key_down(KeyCode::Z) {
            current_animation = 2;
        }

        next_frame().await
    }
}