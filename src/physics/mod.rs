mod constants;
mod helpers;

use std::f32::consts::PI;
use std::ops::DerefMut;
use wrapped2d::b2::{BodyHandle, Joint, JointHandle, UnknownJoint, Vec2};

use constants::*;

/// A headless implementation of QWOP
pub struct QwopPhysics {
    world: helpers::World,
    ground: BodyHandle,
    torso: BodyHandle,
    head: BodyHandle,
    #[allow(unused)]
    left_arm: BodyHandle,
    #[allow(unused)]
    left_forearm: BodyHandle,
    left_thigh: BodyHandle,
    left_calf: BodyHandle,
    left_foot: BodyHandle,
    #[allow(unused)]
    right_arm: BodyHandle,
    #[allow(unused)]
    right_forearm: BodyHandle,
    right_thigh: BodyHandle,
    right_calf: BodyHandle,
    right_foot: BodyHandle,
    #[allow(unused)]
    neck: JointHandle,
    left_shoulder: JointHandle,
    left_hip: JointHandle,
    left_elbow: JointHandle,
    left_knee: JointHandle,
    #[allow(unused)]
    left_ankle: JointHandle,
    right_shoulder: JointHandle,
    right_hip: JointHandle,
    right_elbow: JointHandle,
    right_knee: JointHandle,
    #[allow(unused)]
    right_ankle: JointHandle,
    fall_reset_time: Option<f32>,
}

unsafe impl Send for QwopPhysics {}

impl Default for QwopPhysics {
    #[allow(clippy::excessive_precision)]
    fn default() -> Self {
        let mut world = helpers::World::new(&Vec2 { x: 0.0, y: 10.0 });

        let ground = helpers::create_ground_body(&mut world);

        let torso = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: 2.5111726226000157,
            y: -1.8709517533957938,
            angle: -1.2514497119301329,
            hw: 3.275,
            hh: 1.425,
            friction: 0.2,
            density: 1.0,
            mask_bits: MASK_NO_SELF,
        });
        let head = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: 3.888130278719558,
            y: -5.621802929095265,
            angle: 0.06448415835225099,
            hw: 1.075,
            hh: 1.325,
            friction: 0.2,
            density: 1.0,
            mask_bits: MASK_NO_SELF,
        });
        let left_arm = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: 4.417861014480877,
            y: -2.806563606410589,
            angle: 0.9040095895272826,
            hw: 1.85,
            hh: 0.625,
            friction: 0.2,
            density: 1.0,
            mask_bits: MASK_NONE,
        });
        let left_forearm = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: 5.830008603424893,
            y: -2.8733539631159584,
            angle: -1.2049772618421237,
            hw: 1.75,
            hh: 0.55,
            friction: 0.2,
            density: 1.0,
            mask_bits: MASK_NONE,
        });
        let left_thigh = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: 2.5648987628203876,
            y: 1.648090668682522,
            angle: -2.0177234426823394,
            hw: 2.525,
            hh: 1.0,
            friction: 0.2,
            density: 1.0,
            mask_bits: MASK_NO_SELF,
        });
        let left_calf = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: 3.12585731974087,
            y: 5.525511655361298,
            angle: -1.5903971528225265,
            hw: 2.5,
            hh: 0.75,
            friction: 0.2,
            density: 1.0,
            mask_bits: MASK_NO_SELF,
        });
        let left_foot = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: 3.926921842806667,
            y: 8.08884032049622,
            angle: 0.12027524643408766,
            hw: 1.35,
            hh: 0.675,
            friction: 1.5,
            density: 3.0,
            mask_bits: MASK_NO_SELF,
        });
        let right_arm = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: 1.1812303663272852,
            y: -3.5000256518601014,
            angle: -0.5222217404634386,
            hw: 1.95,
            hh: 0.75,
            friction: 0.2,
            density: 1.0,
            mask_bits: MASK_NONE,
        });
        let right_forearm = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: 0.4078206420797428,
            y: -1.0599953233084172,
            angle: -1.7553358283857299,
            hw: 2.225,
            hh: 0.675,
            friction: 0.2,
            density: 1.0,
            mask_bits: MASK_NONE,
        });
        let right_thigh = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: 1.6120186135678773,
            y: 2.0615320561881516,
            angle: 1.4849422964528027,
            hw: 2.65,
            hh: 1.0,
            friction: 0.2,
            density: 1.0,
            mask_bits: MASK_NO_SELF,
        });
        let right_calf = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: -0.07253905736790486,
            y: 5.347881871063159,
            angle: -0.7588859967104447,
            hw: 2.5,
            hh: 0.75,
            friction: 0.2,
            density: 1.0,
            mask_bits: MASK_NO_SELF,
        });
        let right_foot = helpers::create_player_body(helpers::BodyOptions {
            world: &mut world,
            x: -1.1254742643908706,
            y: 7.567193169625567,
            angle: 0.5897605418219602,
            hw: 1.35,
            hh: 0.725,
            friction: 1.5,
            density: 3.0,
            mask_bits: MASK_NO_SELF,
        });
        let neck = helpers::create_player_joint(helpers::JointOptions {
            world: &mut world,
            body_a: head,
            body_b: torso,
            anchor_a_x: -0.22839485113389058,
            anchor_a_y: 1.1126087775923434,
            anchor_b_x: 2.859519853416241,
            anchor_b_y: 0.1894010834667068,
            lower_angle: -0.5,
            upper_angle: 0.0,
            reference_angle: -1.308996406363529,
            enable_motor: false,
            max_motor_torque: 0.0,
        });
        let left_shoulder = helpers::create_player_joint(helpers::JointOptions {
            world: &mut world,
            body_a: left_arm,
            body_b: torso,
            anchor_a_x: -1.06207890966549,
            anchor_a_y: 0.17409394631566927,
            anchor_b_x: 1.9283425691848985,
            anchor_b_y: 0.5346402981158298,
            lower_angle: -2.0,
            upper_angle: 0.0,
            reference_angle: -2.09438311816829,
            enable_motor: true,
            max_motor_torque: 1000.0,
        });
        let left_hip = helpers::create_player_joint(helpers::JointOptions {
            world: &mut world,
            body_a: left_thigh,
            body_b: torso,
            anchor_a_x: 1.5149934600879298,
            anchor_a_y: 0.10302974517488483,
            anchor_b_x: -2.1617729534350554,
            anchor_b_y: 0.17997450596314002,
            lower_angle: -1.5,
            upper_angle: 0.5,
            reference_angle: 0.7258477508944043,
            enable_motor: true,
            max_motor_torque: 6000.0,
        });
        let left_elbow = helpers::create_player_joint(helpers::JointOptions {
            world: &mut world,
            body_a: left_forearm,
            body_b: left_arm,
            anchor_a_x: -1.2620587423023992,
            anchor_a_y: 0.1572266865964126,
            anchor_b_x: 1.6027887369842988,
            anchor_b_y: -0.1479320438453955,
            lower_angle: -0.1,
            upper_angle: 0.5,
            reference_angle: 2.09438311816829,
            enable_motor: false,
            max_motor_torque: 0.0,
        });
        let left_knee = helpers::create_player_joint(helpers::JointOptions {
            world: &mut world,
            body_a: left_calf,
            body_b: left_thigh,
            anchor_a_x: 2.0031668711363886,
            anchor_a_y: 0.29778450493880393,
            anchor_b_x: -2.039930985466258,
            anchor_b_y: -0.06884320616201567,
            lower_angle: -1.6,
            upper_angle: 0.0,
            reference_angle: -0.3953113764119829,
            enable_motor: true,
            max_motor_torque: 3000.0,
        });
        let left_ankle = helpers::create_player_joint(helpers::JointOptions {
            world: &mut world,
            body_a: left_foot,
            body_b: left_calf,
            anchor_a_x: -0.6270934582479104,
            anchor_a_y: -0.06637286435491153,
            anchor_b_x: -2.425382538900126,
            anchor_b_y: 0.13895539751438726,
            lower_angle: -0.5,
            upper_angle: 0.5,
            reference_angle: -1.7244327585010226,
            enable_motor: false,
            max_motor_torque: 2000.0,
        });
        let right_shoulder = helpers::create_player_joint(helpers::JointOptions {
            world: &mut world,
            body_a: right_arm,
            body_b: torso,
            anchor_a_x: 1.2001841231501342,
            anchor_a_y: 0.014095940491621661,
            anchor_b_x: 2.0154692227269653,
            anchor_b_y: -0.9637164713962119,
            lower_angle: -0.5,
            upper_angle: 1.5,
            reference_angle: -0.7853907065463961,
            enable_motor: true,
            max_motor_torque: 1000.0,
        });
        let right_hip = helpers::create_player_joint(helpers::JointOptions {
            world: &mut world,
            body_a: right_thigh,
            body_b: torso,
            anchor_a_x: -2.0961942262183912,
            anchor_a_y: 0.18536556036297128,
            anchor_b_x: -2.162191390759836,
            anchor_b_y: -0.6165265219145436,
            lower_angle: -1.3,
            upper_angle: 0.7,
            reference_angle: -2.719359381718199,
            enable_motor: true,
            max_motor_torque: 6000.0,
        });
        let right_elbow = helpers::create_player_joint(helpers::JointOptions {
            world: &mut world,
            body_a: right_forearm,
            body_b: right_arm,
            anchor_a_x: 1.786878910753607,
            anchor_a_y: -0.08751611739562593,
            anchor_b_x: -1.3780071636887352,
            anchor_b_y: 0.014064825719665164,
            lower_angle: -0.1,
            upper_angle: 0.5,
            reference_angle: 1.2968199012274688,
            enable_motor: false,
            max_motor_torque: 0.0,
        });
        let right_knee = helpers::create_player_joint(helpers::JointOptions {
            world: &mut world,
            body_a: right_calf,
            body_b: right_thigh,
            anchor_a_x: 1.9464226250774297,
            anchor_a_y: 0.23026118775573856,
            anchor_b_x: 2.0958596632699917,
            anchor_b_y: 0.2946164190567071,
            lower_angle: -1.3,
            upper_angle: 0.3,
            reference_angle: 2.2893406247158676,
            enable_motor: true,
            max_motor_torque: 3000.0,
        });
        let right_ankle = helpers::create_player_joint(helpers::JointOptions {
            world: &mut world,
            body_a: right_foot,
            body_b: right_calf,
            anchor_a_x: -0.7779783144804985,
            anchor_a_y: -0.20811593451266874,
            anchor_b_x: -2.2591139640195594,
            anchor_b_y: 0.08142886510219283,
            lower_angle: -0.5,
            upper_angle: 0.5,
            reference_angle: -1.5708045825942758,
            enable_motor: false,
            max_motor_torque: 2000.0,
        });

        Self {
            world,
            ground,
            torso,
            head,
            left_arm,
            left_forearm,
            left_thigh,
            left_calf,
            left_foot,
            right_arm,
            right_forearm,
            right_thigh,
            right_calf,
            right_foot,
            neck,
            left_shoulder,
            left_hip,
            left_elbow,
            left_knee,
            left_ankle,
            right_shoulder,
            right_hip,
            right_elbow,
            right_knee,
            right_ankle,
            fall_reset_time: None,
        }
    }
}

impl QwopPhysics {
    /// Set the inputs for the physics simulation
    pub fn control(&mut self, q: bool, w: bool, o: bool, p: bool) {
        let set_motor_speed = |joint_handle: JointHandle, speed: f32| {
            let mut meta_joint = self.world.joint_mut(joint_handle);
            if let UnknownJoint::Revolute(joint) = meta_joint.deref_mut() as &mut UnknownJoint {
                joint.set_motor_speed(speed);
            }
        };
        let set_limits = |joint_handle: JointHandle, lower: f32, upper: f32| {
            let mut meta_joint = self.world.joint_mut(joint_handle);
            if let UnknownJoint::Revolute(joint) = meta_joint.deref_mut() as &mut UnknownJoint {
                joint.set_limits(lower, upper);
            }
        };

        let physics_state = &self;

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

    // Advance the physics simulation by the given frame time
    pub fn step(&mut self, frame_time: f32) {
        // Use 2 substeps instead of the 5 that real QWOP uses, since this typically runs at 60 FPS
        // instead of 30
        self.world.step(frame_time * QWOP_TIME_DILATION, 2, 2);

        self.fall_reset_time = match self.fall_reset_time {
            Some(fall_reset_time) => Some(fall_reset_time + frame_time),
            None => {
                // In real QWOP, falling is detected by the head, arms, or forearms touching the
                // track. We use the head or torso instead because the arm poses aren't shown in
                // game.
                if self.world.contacts().any(|contact| {
                    let (body_a, _) = contact.fixture_a();
                    let (body_b, _) = contact.fixture_b();
                    body_a == self.ground && (body_b == self.torso || body_b == self.head)
                }) {
                    Some(0.0)
                } else {
                    None
                }
            }
        };

        // Reset after ragdolling for a short period of time
        if self.fall_reset_time.is_some_and(|time| time > RESET_TIME) {
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn root_angle(&self) -> f32 {
        let torso = self.world.body(self.torso);
        -torso.angle() - PI / 2.0
    }

    pub fn neck_angle(&self) -> f32 {
        let head = self.world.body(self.head);
        let torso = self.world.body(self.torso);
        -(head.angle() - torso.angle() - PI / 2.0)
    }

    pub fn left_hip_angle(&self) -> f32 {
        let torso = self.world.body(self.torso);
        let left_thigh = self.world.body(self.left_thigh);
        left_thigh.angle() - torso.angle()
    }

    pub fn right_hip_angle(&self) -> f32 {
        let torso = self.world.body(self.torso);
        let right_thigh = self.world.body(self.right_thigh);
        right_thigh.angle() - torso.angle() + PI
    }

    pub fn left_knee_angle(&self) -> f32 {
        let left_calf = self.world.body(self.left_calf);
        let left_thigh = self.world.body(self.left_thigh);
        left_calf.angle() - left_thigh.angle()
    }

    pub fn right_knee_angle(&self) -> f32 {
        let right_calf = self.world.body(self.right_calf);
        let right_thigh = self.world.body(self.right_thigh);
        right_calf.angle() - right_thigh.angle() + PI
    }

    pub fn left_foot_angle(&self) -> f32 {
        let left_calf = self.world.body(self.left_calf);
        let left_foot = self.world.body(self.left_foot);
        left_foot.angle() - left_calf.angle() - PI / 2.0
    }

    pub fn right_foot_angle(&self) -> f32 {
        let right_calf = self.world.body(self.right_calf);
        let right_foot = self.world.body(self.right_foot);
        right_foot.angle() - right_calf.angle() - PI / 2.0
    }

    pub fn elevation(&self) -> f32 {
        let left_hip = self.world.joint(self.left_hip);
        (WORLD_HEIGHT - left_hip.anchor_a().y - 10.0) / QWOP_TO_METERS_SCALE
    }

    pub fn distance(&self) -> f32 {
        let left_hip = self.world.joint(self.left_hip);
        (left_hip.anchor_a().x - 1.5149934600879298) / QWOP_TO_METERS_SCALE
    }

    pub fn velocity(&self) -> f32 {
        let torso = self.world.body(self.torso);
        let left_hip = self.world.joint(self.left_hip);
        let linear_velocity = torso.linear_velocity_from_world_point(&left_hip.anchor_a());
        linear_velocity.x / QWOP_TO_METERS_SCALE
    }

    pub fn fallen(&self) -> bool {
        self.fall_reset_time.is_some()
    }

    pub fn just_fallen(&self) -> bool {
        self.fall_reset_time == Some(0.0)
    }
}
