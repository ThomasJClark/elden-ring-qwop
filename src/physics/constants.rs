use wrapped2d::b2::Vec2;

pub const WORLD_WIDTH: f32 = 99999.0;
pub const WORLD_HEIGHT: f32 = 20.0;
pub const CATEGORY_GROUND: u16 = 0x0001;
pub const CATEGORY_PLAYER: u16 = 0x0002;
pub const MASK_NO_SELF: u16 = 0xfffd;
pub const MASK_ALL: u16 = 0xffff;
pub const GROUND_HALF_WIDTH: f32 = 10.0 * WORLD_WIDTH;
pub const GROUND_HALF_HEIGHT: f32 = 0.5;
pub const QWOP_TO_WORLD_SCALE: f32 = 9.0;
pub const INITIAL_POSITION_OFFSET: Vec2 = Vec2 { x: 0.0, y: 9.0 };
pub const RESET_TIME: f32 = 1.5;

/// QWOP runs at 30 FPS, but the Box2D physics world is updated by 40 ms per frame. Speed up time by
/// this ratio to preserve speed of real QWOP
pub const QWOP_TIME_DILATION: f32 = 30.0 * 0.04;
