use wrapped2d::b2::{
    BodyDef, BodyHandle, BodyType, Filter, FixtureDef, JointHandle, PolygonShape, RevoluteJointDef,
    Vec2,
};

use crate::physics::constants::*;

pub type World = wrapped2d::b2::World<wrapped2d::user_data::NoUserData>;

pub fn create_ground_body(world: &mut World) -> BodyHandle {
    let body_handle = world.create_body(&BodyDef {
        body_type: BodyType::Static,
        position: Vec2 {
            x: WORLD_WIDTH / 2.0,
            y: WORLD_HEIGHT - GROUND_HALF_HEIGHT,
        },
        ..BodyDef::new()
    });

    world.body_mut(body_handle).create_fixture(
        &PolygonShape::new_box(GROUND_HALF_WIDTH, GROUND_HALF_HEIGHT),
        &mut FixtureDef {
            friction: 0.2,
            density: 20.0,
            filter: Filter {
                category_bits: CATEGORY_GROUND,
                mask_bits: MASK_ALL,
                ..Filter::new()
            },
            ..FixtureDef::new()
        },
    );

    body_handle
}

pub struct BodyOptions<'a> {
    pub world: &'a mut World,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
    pub hw: f32,
    pub hh: f32,
    pub friction: f32,
    pub density: f32,
    pub mask_bits: u16,
}

pub fn create_player_body(
    BodyOptions {
        world,
        x,
        y,
        angle,
        hw,
        hh,
        friction,
        density,
        mask_bits,
    }: BodyOptions,
) -> BodyHandle {
    let body_handle = world.create_body(&BodyDef {
        body_type: BodyType::Dynamic,
        position: Vec2 { x, y } + INITIAL_POSITION_OFFSET,
        angle,
        ..BodyDef::new()
    });

    world.body_mut(body_handle).create_fixture(
        &PolygonShape::new_box(hw, hh),
        &mut FixtureDef {
            friction,
            density,
            filter: Filter {
                category_bits: CATEGORY_PLAYER,
                mask_bits,
                ..Filter::new()
            },
            ..FixtureDef::new()
        },
    );

    body_handle
}

pub struct JointOptions<'a> {
    pub world: &'a mut World,
    pub body_a: BodyHandle,
    pub body_b: BodyHandle,
    pub anchor_a_x: f32,
    pub anchor_a_y: f32,
    pub anchor_b_x: f32,
    pub anchor_b_y: f32,
    pub lower_angle: f32,
    pub upper_angle: f32,
    pub reference_angle: f32,
    pub enable_motor: bool,
    pub max_motor_torque: f32,
}

pub fn create_player_joint(
    JointOptions {
        world,
        body_a,
        body_b,
        anchor_a_x,
        anchor_a_y,
        anchor_b_x,
        anchor_b_y,
        lower_angle,
        upper_angle,
        reference_angle,
        enable_motor,
        max_motor_torque,
    }: JointOptions,
) -> JointHandle {
    world.create_joint(&RevoluteJointDef {
        local_anchor_a: Vec2 {
            x: anchor_a_x,
            y: anchor_a_y,
        },
        local_anchor_b: Vec2 {
            x: anchor_b_x,
            y: anchor_b_y,
        },
        reference_angle,
        enable_limit: true,
        lower_angle,
        upper_angle,
        enable_motor,
        max_motor_torque,
        ..RevoluteJointDef::new(body_a, body_b)
    })
}
