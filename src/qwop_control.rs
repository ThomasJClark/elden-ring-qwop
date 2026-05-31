use std::ops::DerefMut;
use wrapped2d::b2::{JointHandle, UnknownJoint};

use crate::qwop_physics::QwopPhysics;

const RESET_TIME: f32 = 1.0;

pub struct QwopControl {
    pub physics: QwopPhysics,
    pub just_fallen: bool,
    pub fallen: bool,
    pub reset_timer: f32,
}

unsafe impl Send for QwopControl {}

impl QwopControl {
    pub fn new() -> Self {
        Self {
            physics: QwopPhysics::new(),
            just_fallen: false,
            fallen: false,
            reset_timer: -1.0,
        }
    }

    pub fn control(&mut self, q: bool, w: bool, o: bool, p: bool) {
        let set_motor_speed = |joint_handle: JointHandle, speed: f32| {
            let mut meta_joint = self.physics.world.joint_mut(joint_handle);
            if let UnknownJoint::Revolute(joint) = meta_joint.deref_mut() as &mut UnknownJoint {
                joint.set_motor_speed(speed);
            }
        };
        let set_limits = |joint_handle: JointHandle, lower: f32, upper: f32| {
            let mut meta_joint = self.physics.world.joint_mut(joint_handle);
            if let UnknownJoint::Revolute(joint) = meta_joint.deref_mut() as &mut UnknownJoint {
                joint.set_limits(lower, upper);
            }
        };

        let physics_state = &self.physics;

        if q {
            set_motor_speed(physics_state.right_hip, 2.5);
            set_motor_speed(physics_state.left_hip, -2.5);
            set_motor_speed(physics_state.right_shoulder, -2.0);
            set_motor_speed(physics_state.right_elbow, -10.0);
            set_motor_speed(physics_state.left_shoulder, 2.0);
            set_motor_speed(physics_state.left_elbow, -10.0);
        } else if w {
            set_motor_speed(physics_state.right_hip, -2.5);
            set_motor_speed(physics_state.left_hip, 2.5);
            set_motor_speed(physics_state.right_shoulder, 2.0);
            set_motor_speed(physics_state.left_shoulder, -2.0);
            set_motor_speed(physics_state.right_elbow, 10.0);
            set_motor_speed(physics_state.left_elbow, 10.0);
        } else {
            set_motor_speed(physics_state.right_hip, 0.0);
            set_motor_speed(physics_state.left_hip, 0.0);
            set_motor_speed(physics_state.left_shoulder, 0.0);
            set_motor_speed(physics_state.right_shoulder, 0.0);
        }
        if o {
            set_motor_speed(physics_state.right_knee, 2.5);
            set_motor_speed(physics_state.left_knee, -2.5);
            set_limits(physics_state.left_hip, -1.0, 1.0);
            set_limits(physics_state.right_hip, -1.3, 0.7);
        } else if p {
            set_motor_speed(physics_state.right_knee, -2.5);
            set_motor_speed(physics_state.left_knee, 2.5);
            set_limits(physics_state.left_hip, -1.5, 0.5);
            set_limits(physics_state.right_hip, -0.8, 1.2);
        } else {
            set_motor_speed(physics_state.right_knee, 0.0);
            set_motor_speed(physics_state.left_knee, 0.0);
        }
    }

    pub fn step(&mut self, frame_time: f32) {
        // QWOP uses 5 substeps at 25 fps. 2 substeps at 60 fps is approximately the same update
        // rate
        self.physics.world.step(frame_time, 2, 2);

        // Reset after ragdolling for 1 second. `self.just_fallen` in dicates that we just fell this
        // frame, so a single instance of damage can be applied
        if self.reset_timer < 0.0 {
            if self.physics.fallen() {
                self.fallen = true;
                self.just_fallen = true;
                self.reset_timer = 0.0;
            }
        } else {
            self.just_fallen = false;
            self.reset_timer += frame_time;
            if self.reset_timer > RESET_TIME {
                self.reset();
            }
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
