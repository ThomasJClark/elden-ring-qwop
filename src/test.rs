mod qwop;

use std::cell::Ref;

use macroquad::prelude::*;
use wrapped2d::{
    b2::{self, Joint, MetaBody, MetaJoint},
    user_data::NoUserData,
};

use crate::qwop::{GROUND_HALF_HEIGHT, GROUND_HALF_WIDTH};

const WORLD_SCALE: f32 = 20.0;
const WORLD_OFFSET: b2::Vec2 = b2::Vec2 { x: 10.0, y: 10.0 };

fn draw_body(body: Ref<'_, MetaBody<NoUserData>>, hw: f32, hh: f32, color: Color) {
    let position = (body.position() + WORLD_OFFSET) * WORLD_SCALE;

    draw_rectangle_ex(
        position.x,
        position.y,
        2.0 * hw * WORLD_SCALE,
        2.0 * hh * WORLD_SCALE,
        DrawRectangleParams {
            color,
            offset: vec2(0.5, 0.5),
            rotation: body.angle(),
        },
    );
}

fn draw_joint(joint: Ref<'_, MetaJoint<NoUserData>>) {
    let anchor_a = (joint.anchor_a() + WORLD_OFFSET) * WORLD_SCALE;
    let anchor_b = (joint.anchor_b() + WORLD_OFFSET) * WORLD_SCALE;

    draw_line(anchor_a.x, anchor_a.y, anchor_b.x, anchor_b.y, 2.0, RED);
    draw_circle_lines(anchor_a.x, anchor_a.y, 0.25 * WORLD_SCALE, 2.0, BLUE);
    draw_circle_lines(anchor_b.x, anchor_b.y, 0.25 * WORLD_SCALE, 2.0, GREEN);
}

#[macroquad::main("qwop test")]
async fn main() {
    let mut qwop = qwop::Qwop::new();

    loop {
        if is_key_down(KeyCode::F4) {
            qwop.reset();
        } else {
            qwop.control(
                is_key_down(KeyCode::Q),
                is_key_down(KeyCode::W),
                is_key_down(KeyCode::O),
                is_key_down(KeyCode::P),
            );
        }

        let physics = &qwop.physics_state;

        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

        draw_body(
            physics.world.body(physics.ground),
            GROUND_HALF_WIDTH,
            GROUND_HALF_HEIGHT,
            Color::from_hex(0x2c3e50ff),
        );
        draw_body(
            physics.world.body(physics.left_calf),
            2.5,
            0.75,
            Color::from_rgba(110, 75, 50, 255),
        );
        draw_body(
            physics.world.body(physics.left_foot),
            1.35,
            0.675,
            Color::from_rgba(50, 50, 50, 255),
        );
        draw_body(
            physics.world.body(physics.left_thigh),
            2.525,
            1.0,
            Color::from_rgba(90, 60, 40, 255),
        );
        draw_body(
            physics.world.body(physics.left_arm),
            1.85,
            0.625,
            Color::from_rgba(70, 100, 140, 255),
        );
        draw_body(
            physics.world.body(physics.left_forearm),
            1.75,
            0.55,
            Color::from_rgba(80, 110, 150, 255),
        );
        draw_body(
            physics.world.body(physics.torso),
            3.275,
            1.425,
            Color::from_rgba(0, 128, 128, 255),
        );
        draw_body(
            physics.world.body(physics.head),
            1.075,
            1.325,
            Color::from_rgba(245, 222, 179, 255),
        );
        draw_body(
            physics.world.body(physics.right_arm),
            1.95,
            0.75,
            Color::from_rgba(200, 120, 80, 255),
        );
        draw_body(
            physics.world.body(physics.right_calf),
            2.5,
            0.75,
            Color::from_rgba(180, 115, 85, 255),
        );
        draw_body(
            physics.world.body(physics.right_thigh),
            2.65,
            1.0,
            Color::from_rgba(160, 100, 70, 255),
        );
        draw_body(
            physics.world.body(physics.right_foot),
            1.35,
            0.725,
            Color::from_rgba(60, 60, 60, 255),
        );
        draw_body(
            physics.world.body(physics.right_forearm),
            2.225,
            0.675,
            Color::from_rgba(220, 140, 100, 255),
        );
        draw_joint(physics.world.joint(physics.neck));
        draw_joint(physics.world.joint(physics.left_shoulder));
        draw_joint(physics.world.joint(physics.left_hip));
        draw_joint(physics.world.joint(physics.left_elbow));
        draw_joint(physics.world.joint(physics.left_knee));
        draw_joint(physics.world.joint(physics.left_ankle));
        draw_joint(physics.world.joint(physics.right_shoulder));
        draw_joint(physics.world.joint(physics.right_hip));
        draw_joint(physics.world.joint(physics.right_elbow));
        draw_joint(physics.world.joint(physics.right_knee));
        draw_joint(physics.world.joint(physics.right_ankle));

        qwop.step(get_frame_time());

        next_frame().await
    }
}
