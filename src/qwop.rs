use std::{f32::consts::PI, ops::DerefMut};
use wrapped2d::b2::{
    BodyDef, BodyHandle, BodyType, Filter, FixtureDef, Joint, JointHandle, PolygonShape,
    RevoluteJointDef, UnknownJoint, Vec2,
};

type World = wrapped2d::b2::World<wrapped2d::user_data::NoUserData>;

const WORLD_WIDTH: f32 = 32.0;
const WORLD_HEIGHT: f32 = 20.0;
const CATEGORY_GROUND: u16 = 0x0001;
const CATEGORY_PLAYER: u16 = 0x0002;
const MASK_NO_SELF: u16 = 0xfffd;
const MASK_ALL: u16 = 0xffff;
const GROUND_HALF_WIDTH: f32 = 10.0 * WORLD_WIDTH;
const GROUND_HALF_HEIGHT: f32 = 0.5;

pub struct QwopPhysicsState {
    world: World,
    ground: BodyHandle,
    torso: BodyHandle,
    head: BodyHandle,
    left_arm: BodyHandle,
    left_forearm: BodyHandle,
    left_thigh: BodyHandle,
    left_calf: BodyHandle,
    left_foot: BodyHandle,
    right_arm: BodyHandle,
    right_forearm: BodyHandle,
    right_thigh: BodyHandle,
    right_calf: BodyHandle,
    right_foot: BodyHandle,
    neck: JointHandle,
    left_shoulder: JointHandle,
    left_hip: JointHandle,
    left_elbow: JointHandle,
    left_knee: JointHandle,
    left_ankle: JointHandle,
    right_shoulder: JointHandle,
    right_hip: JointHandle,
    right_elbow: JointHandle,
    right_knee: JointHandle,
    right_ankle: JointHandle,
}

/// Headless reimplementation of the QWOP physics engine
pub struct Qwop {
    pub physics_state: QwopPhysicsState,
}

#[derive(Debug)]
pub struct SkeletonState {
    pub elevation: f32,
    pub root: f32,
    pub left_hip: f32,
    pub right_hip: f32,
    pub left_knee: f32,
    pub right_knee: f32,
    pub left_foot: f32,
    pub right_foot: f32,
}

unsafe impl Send for Qwop {}

fn create_ground_body(world: &mut World) -> BodyHandle {
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

struct BodyOptions<'a> {
    world: &'a mut World,
    x: f32,
    y: f32,
    angle: f32,
    hw: f32,
    hh: f32,
    friction: f32,
    density: f32,
}

fn create_player_body(
    BodyOptions {
        world,
        x,
        y,
        angle,
        hw,
        hh,
        friction,
        density,
    }: BodyOptions,
) -> BodyHandle {
    let body_handle = world.create_body(&BodyDef {
        body_type: BodyType::Dynamic,
        position: Vec2 { x, y },
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
                mask_bits: MASK_NO_SELF,
                ..Filter::new()
            },
            ..FixtureDef::new()
        },
    );

    body_handle
}

struct JointOptions<'a> {
    world: &'a mut World,
    body_a: BodyHandle,
    body_b: BodyHandle,
    anchor_a_x: f32,
    anchor_a_y: f32,
    anchor_b_x: f32,
    anchor_b_y: f32,
    lower_angle: f32,
    upper_angle: f32,
    reference_angle: f32,
    enable_motor: bool,
    max_motor_torque: f32,
}

fn create_player_joint(
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
    let local_anchor_a = world.body(body_a).local_point(&Vec2 {
        x: anchor_a_x,
        y: anchor_a_y,
    });

    let local_anchor_b = world.body(body_b).local_point(&Vec2 {
        x: anchor_b_x,
        y: anchor_b_y,
    });

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

impl QwopPhysicsState {
    #[allow(clippy::excessive_precision)]
    fn new() -> Self {
        let mut world = World::new(&Vec2 { x: 0.0, y: 10.0 });

        let ground = create_ground_body(&mut world);

        let torso = create_player_body(BodyOptions {
            world: &mut world,
            x: 2.5111726226000157,
            y: -1.8709517533957938,
            angle: -1.2514497119301329,
            hw: 3.275,
            hh: 1.425,
            friction: 0.2,
            density: 1.0,
        });
        let head = create_player_body(BodyOptions {
            world: &mut world,
            x: 3.888130278719558,
            y: -5.621802929095265,
            angle: 0.06448415835225099,
            hw: 1.075,
            hh: 1.325,
            friction: 0.2,
            density: 1.0,
        });
        let left_arm = create_player_body(BodyOptions {
            world: &mut world,
            x: 4.417861014480877,
            y: -2.806563606410589,
            angle: 0.9040095895272826,
            hw: 1.85,
            hh: 0.625,
            friction: 0.2,
            density: 1.0,
        });
        let left_forearm = create_player_body(BodyOptions {
            world: &mut world,
            x: 5.830008603424893,
            y: -2.8733539631159584,
            angle: -1.2049772618421237,
            hw: 1.75,
            hh: 0.55,
            friction: 0.2,
            density: 1.0,
        });
        let left_thigh = create_player_body(BodyOptions {
            world: &mut world,
            x: 2.5648987628203876,
            y: 1.648090668682522,
            angle: -2.0177234426823394,
            hw: 2.525,
            hh: 1.0,
            friction: 0.2,
            density: 1.0,
        });
        let left_calf = create_player_body(BodyOptions {
            world: &mut world,
            x: 3.12585731974087,
            y: 5.525511655361298,
            angle: -1.5903971528225265,
            hw: 2.5,
            hh: 0.75,
            friction: 0.2,
            density: 1.0,
        });
        let left_foot = create_player_body(BodyOptions {
            world: &mut world,
            x: 3.926921842806667,
            y: 8.08884032049622,
            angle: 0.12027524643408766,
            hw: 1.35,
            hh: 0.675,
            friction: 1.5,
            density: 3.0,
        });
        let right_arm = create_player_body(BodyOptions {
            world: &mut world,
            x: 1.1812303663272852,
            y: -3.5000256518601014,
            angle: -0.5222217404634386,
            hw: 1.95,
            hh: 0.75,
            friction: 0.2,
            density: 1.0,
        });
        let right_forearm = create_player_body(BodyOptions {
            world: &mut world,
            x: 0.4078206420797428,
            y: -1.0599953233084172,
            angle: -1.7553358283857299,
            hw: 2.225,
            hh: 0.675,
            friction: 0.2,
            density: 1.0,
        });
        let right_thigh = create_player_body(BodyOptions {
            world: &mut world,
            x: 1.6120186135678773,
            y: 2.0615320561881516,
            angle: 1.4849422964528027,
            hw: 2.65,
            hh: 1.0,
            friction: 0.2,
            density: 1.0,
        });
        let right_calf = create_player_body(BodyOptions {
            world: &mut world,
            x: -0.07253905736790486,
            y: 5.347881871063159,
            angle: -0.7588859967104447,
            hw: 2.5,
            hh: 0.75,
            friction: 0.2,
            density: 1.0,
        });
        let right_foot = create_player_body(BodyOptions {
            world: &mut world,
            x: -1.1254742643908706,
            y: 7.567193169625567,
            angle: 0.5897605418219602,
            hw: 1.35,
            hh: 0.725,
            friction: 1.5,
            density: 3.0,
        });
        let neck = create_player_joint(JointOptions {
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
        let left_shoulder = create_player_joint(JointOptions {
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
        let left_hip = create_player_joint(JointOptions {
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
        let left_elbow = create_player_joint(JointOptions {
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
        let left_knee = create_player_joint(JointOptions {
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
        let left_ankle = create_player_joint(JointOptions {
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
        let right_shoulder = create_player_joint(JointOptions {
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
        let right_hip = create_player_joint(JointOptions {
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
        let right_elbow = create_player_joint(JointOptions {
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
        let right_knee = create_player_joint(JointOptions {
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
        let right_ankle = create_player_joint(JointOptions {
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
        }
    }
}

impl Qwop {
    pub fn new() -> Self {
        Self {
            physics_state: QwopPhysicsState::new(),
        }
    }

    pub fn control(&mut self, q: bool, w: bool, o: bool, p: bool) {
        let set_motor_speed = |joint_handle: JointHandle, speed: f32| {
            let mut meta_joint = self.physics_state.world.joint_mut(joint_handle);
            if let UnknownJoint::Revolute(joint) = meta_joint.deref_mut() as &mut UnknownJoint {
                joint.set_motor_speed(speed);
            }
        };
        let set_limits = |joint_handle: JointHandle, lower: f32, upper: f32| {
            let mut meta_joint = self.physics_state.world.joint_mut(joint_handle);
            if let UnknownJoint::Revolute(joint) = meta_joint.deref_mut() as &mut UnknownJoint {
                joint.set_limits(lower, upper);
            }
        };

        let physics_state = &self.physics_state;

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

    pub fn reset(&mut self) {
        self.physics_state = QwopPhysicsState::new();
    }

    pub fn step(&mut self, frame_time: f32) {
        self.physics_state.world.step(frame_time, 5, 5);
    }

    pub fn velocity(&self) -> f32 {
        let world = &self.physics_state.world;
        let left_hip = self.physics_state.left_hip;
        let torso = self.physics_state.torso;

        let linear_velocity = world
            .body(torso)
            .linear_velocity_from_world_point(&world.joint(left_hip).anchor_a());

        linear_velocity.x / 9.0
    }

    pub fn skeleton(&self) -> SkeletonState {
        let world = &self.physics_state.world;
        let torso = world.body(self.physics_state.torso);
        let left_calf = world.body(self.physics_state.left_calf);
        let left_foot = world.body(self.physics_state.left_foot);
        let left_thigh = world.body(self.physics_state.left_thigh);
        let right_calf = world.body(self.physics_state.right_calf);
        let right_foot = world.body(self.physics_state.right_foot);
        let right_thigh = world.body(self.physics_state.right_thigh);
        let left_hip = world.joint(self.physics_state.left_hip);

        SkeletonState {
            root: -torso.angle() - PI / 2.0,
            left_hip: left_thigh.angle() - torso.angle(),
            right_hip: right_thigh.angle() - torso.angle() + PI,
            left_knee: left_calf.angle() - left_thigh.angle(),
            right_knee: right_calf.angle() - right_thigh.angle() + PI,
            left_foot: left_foot.angle() - left_calf.angle() - PI / 2.0,
            right_foot: right_foot.angle() - right_calf.angle() - PI / 2.0,
            elevation: (WORLD_HEIGHT - left_hip.anchor_a().y - 10.0) / 9.0,
        }
    }
}
