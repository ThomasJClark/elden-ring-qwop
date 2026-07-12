use core::f32;
use std::ptr::NonNull;

use eldenring::cs::{
    CSCamera, CSFeManImp, ChrCtrl, ChrInsExt, EquipParamGoods, MenuString, SoloParamRepository,
    ThrowNodeState, WorldChrMan,
};
use eldenring::dlkr::DLAllocator;
use eldenring::dltx::DLString;
use eldenring::havok::HkQuaternion;
use fromsoftware_shared::FromStatic;
use glam::{Vec3, Vec4, Vec4Swizzles};

use crate::input_state::QwopInputState;
use crate::physics::QwopPhysics;
use crate::player_ins_skeleton_ext::PlayerInsSkeletonExt;

/// SpEffectParam that applies damage and blood splatter VFX after falling
const FALLEN_SP_EFFECT_ID: i32 = 67;

/// SpEffectParam active while the player is resting at a grace. QWOP is disabled in this state.
const SITE_OF_LOST_GRACE_SP_EFFECT_ID: i32 = 9607;

/// EquipParamGoods for the horse summon wistle. Horse is banned because it trivializes movement
const SPECTRAL_STEED_WHISTLE_GOODS_ID: u32 = 130;

const ROTATION_SPEED: f32 = 540.0_f32.to_radians();

/// Time in seconds to visually transition between QWOP enabled and disabled states
const TRANSITION_TIME: f32 = 0.35;

const ROOT_MOTION_DAMPING: f32 = 4.0;

// Conversion factor for QWOP meters to Elden Ring physics units determined by vibes
const WORLD_SCALE: f32 = 1.1165984;

#[derive(Default)]
pub struct QwopMod {
    input_state: QwopInputState,
    physics: QwopPhysics,
    prev_main_player_loaded: bool,
    displacement: Vec3,
    qwop_enabled: bool,
    transition_time: f32,
    cheese_discovered: bool,
}

unsafe impl Sync for QwopMod {}

/// Top level mod logic for the QWOP mod. This is just responsible for gluing together the QWOP
/// state and the Elden Ring state at various hooks and tasks.
impl QwopMod {
    /// The main update loop for the QWOP mod. Polls for input, advances the QWOP physics
    /// simulation, and updates any game state that doesn't need to be a in a specific hook or task
    /// group to avoid getting overwritten.
    pub fn chr_ins_pre_behavior(&mut self) {
        let Some(player) = unsafe { WorldChrMan::instance_mut().ok() }
            .and_then(|world_chr_man| world_chr_man.main_player.as_mut())
        else {
            self.prev_main_player_loaded = false;
            return;
        };

        // Reset the physics simulation when the world is reloaded so the player starts in the
        // default pose when loading in
        if !self.prev_main_player_loaded {
            self.physics.reset();
            self.prev_main_player_loaded = true;
            self.displacement = Vec3::ZERO;
            self.transition_time = 0.0;
        }

        self.input_state.poll();

        self.qwop_enabled =
            // Disable QWOP if toggled off by the player
            !self.input_state.disabled
            // Disable QWOP while performing a crit or being critted
            && player.modules.throw.throw_node.throw_state == ThrowNodeState::None
            // Disable QWOP while resting at a bonfire
            && !player
                .special_effect
                .entries()
                .any(|entry| entry.param_id == SITE_OF_LOST_GRACE_SP_EFFECT_ID);

        if self.qwop_enabled {
            self.physics.control(
                self.input_state.q,
                self.input_state.w,
                self.input_state.o,
                self.input_state.p,
            );

            self.physics.step(player.modules.hitstop.frame_time);

            let distance = self.physics.distance();

            // Apply damage and reset the fallen flag when the player falls
            if self.physics.just_fallen() {
                if let Ok(fe_man) = unsafe { CSFeManImp::instance_mut() } {
                    fe_man.frontend_values.area_welcome_message = MenuString {
                        static_string: std::ptr::null(),
                        allocated_string: DLString::from_str(
                            format!(
                                "You ran <font color=\"{}\">{:.1} meters</font>",
                                if distance >= 0.0 {
                                    "#ffcc33"
                                } else {
                                    "#770d17"
                                },
                                distance
                            ),
                            DLAllocator::runtime_heap_allocator(),
                        )
                        .unwrap(),
                    };
                }

                player.apply_speffect(FALLEN_SP_EFFECT_ID, true);
            }

            // Running backwards is a bit of a cheese but I think it's more fun to not disallow it.
            // Let the player know we're on to them.
            if !self.cheese_discovered && distance < -50.0 {
                if let Ok(fe_man) = unsafe { CSFeManImp::instance_mut() } {
                    fe_man.frontend_values.full_screen_message_request_id =
                        eldenring::cs::FullScreenMessage::HunterRankAdvanced;
                    self.cheese_discovered = true;
                }
            }

            self.transition_time =
                (self.transition_time + player.modules.hitstop.frame_time).min(TRANSITION_TIME);
        } else {
            self.transition_time =
                (self.transition_time - player.modules.hitstop.frame_time).max(0.0);
        }

        // No normal walking, sneaking, rolling, backstep, or horse allowed while QWOP is enabled.
        let disabled_actions = &mut player.modules.action_request.disabled_action_inputs;
        disabled_actions.set_l3(self.qwop_enabled);
        disabled_actions.set_rolling(self.qwop_enabled);
        disabled_actions.set_backstep(self.qwop_enabled);
        player.debug_flags.set_disabled_movement(self.qwop_enabled);

        if let Some(horse_whistle) = unsafe { SoloParamRepository::instance_mut() }
            .ok()
            .and_then(|solo_param_repository| {
                solo_param_repository.get_mut::<EquipParamGoods>(SPECTRAL_STEED_WHISTLE_GOODS_ID)
            })
        {
            horse_whistle.set_enable_live(!self.qwop_enabled)
        }
    }

    /// Update the player's root motion based on the current QWOP physics state. This must be done
    /// in a hook in the middle of the HavokBehavior task group, after the player's root motion has
    /// been set but before it is applied.
    pub fn chr_ctrl_update_pos_hook(&mut self, chr_ctrl: NonNull<ChrCtrl>) {
        if !self.qwop_enabled {
            return;
        }

        let Some(player) = unsafe { WorldChrMan::instance_mut().ok() }
            .and_then(|world_chr_man| world_chr_man.main_player.as_mut())
            .filter(|player| player.chr_ctrl.as_ptr() == chr_ctrl.as_ptr())
        else {
            return;
        };

        let frame_time = player.modules.hitstop.frame_time;
        let orientation = player.modules.physics.orientation;

        if let Ok(cs_camera) = unsafe { CSCamera::instance() } {
            let target_angle = {
                let camera_matrix = cs_camera.pers_cam_2.matrix;
                let camera_angle = f32::atan2(camera_matrix.0.2, camera_matrix.2.2);

                if player.is_locked_on {
                    // Face the target enemy while locked on
                    Some(camera_angle + 180.0_f32.to_radians())
                } else if self.input_state.q
                    || self.input_state.w
                    || self.input_state.o
                    || self.input_state.p
                {
                    // Face to the right while moving and not locked on (like the 2D side-scrolling
                    // view in the original QWOP)
                    Some(camera_angle + 90.0_f32.to_radians())
                } else {
                    None
                }
            };

            if let Some(target_angle) = target_angle {
                let target_rotation = HkQuaternion::from_rotation_y(-target_angle);

                player.modules.physics.orientation =
                    orientation.rotate_towards(target_rotation, frame_time * ROTATION_SPEED);
            }
        }

        // Keep track of the cumulative change in world coordinates. The total position is dampened
        // over time so that animations can cause instantaneous motion but snap back to the original
        // position. This allows things like stepping forward during an attack or AoW to increase
        // reach, but prevents the player from using this to move long distances without using QWOP
        // controls.
        let local_displacement = orientation
            .inverse()
            .mul_vec3(self.displacement)
            .extend(0.0);
        player.modules.behavior.root_motion -=
            frame_time * ROOT_MOTION_DAMPING * local_displacement;
        self.displacement += orientation
            .mul_vec3(player.modules.behavior.root_motion.xyz())
            .with_y(0.0);

        player.modules.behavior.root_motion +=
            frame_time * -self.physics.velocity() * WORLD_SCALE * Vec4::Z;
    }

    /// Update the main player's skeleton pose based on the current QWOP physics state
    pub fn chr_ins_behavior_safe_hook(&self, mut world_chr_man: NonNull<WorldChrMan>) {
        let Some(player) = &mut unsafe { world_chr_man.as_mut() }.main_player else {
            return;
        };

        player.set_ragdoll(false);

        // When the player falls per QWOP rules or dies in real life, ragdoll because it's funny
        if self.qwop_enabled
            && (self.physics.fallen()
                || unsafe { player.player_game_data.as_ref() }.current_hp == 0)
        {
            player.set_ragdoll(true);
            return;
        }

        let lerp_amount =
            1.0 - f32::cos(self.transition_time / TRANSITION_TIME * f32::consts::PI / 2.0);

        player.set_pose(
            self.physics.elevation() * WORLD_SCALE,
            self.physics.root_angle(),
            self.physics.neck_angle(),
            self.physics.left_hip_angle(),
            self.physics.right_hip_angle(),
            self.physics.left_knee_angle(),
            self.physics.right_knee_angle(),
            self.physics.left_foot_angle(),
            self.physics.right_foot_angle(),
            lerp_amount,
        );
    }
}
