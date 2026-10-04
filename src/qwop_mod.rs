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
use crate::param::FALLEN_SP_EFFECT_ID;
use crate::physics::QwopPhysics;
use crate::player_ins_skeleton_ext::PlayerInsSkeletonExt;

/// EquipParamGoods for the horse summon wistle. Horse is banned because it trivializes movement
const SPECTRAL_STEED_WHISTLE_GOODS_ID: u32 = 130;

const ROTATION_SPEED: f32 = 540.0_f32.to_radians();

const ROOT_MOTION_DAMPING: f32 = 4.0;

// Conversion factor for QWOP meters to Elden Ring physics units determined by vibes
const WORLD_SCALE: f32 = 1.1165984;

static EXCLUDED_ANIMATIONS: phf::Set<i32> = phf::phf_set! {
    60200i32,           // pull lever
    68011i32, 68012i32, // grace
    68021i32, 68022i32, // grace
    60470i32,           // walking through magic portal
    60060i32,           // fog wall
    28030i32, 28040i32, 28011i32, 28012i32, 28021i32, 28022i32, // ladder
    81000i32, 81001i32, // prayer
    81010i32, 81011i32, // desparate prayer
    80400i32,           // extreme repentance
    80410i32,           // grovel for mercy
    80910i32, 80911i32, // crossed legs
    80940i32, 80941i32, // dozing crossed legs
    80920i32, 80921i32, // rest
    80930i32, 80931i32, // sitting sideways
    80800i32, 80801i32, // dejection
    80900i32, 80901i32, // patches crouch
    80970i32, 80971i32, // balled up
    80950i32, 80951i32, // spread out
};

static SLOW_TRANSITION_ANIMATIONS: phf::Set<i32> = phf::phf_set! {
    81000i32, 81002i32, // prayer
    81010i32, 81012i32, // desparate prayer
    80910i32, 80912i32, // crossed legs
    80940i32, 80942i32, // dozing crossed legs
    80920i32, 80922i32, // rest
    80930i32, 80932i32, // sitting sideways
    80800i32, 80802i32, // dejection
    80900i32, 80902i32, // patches crouch
    80970i32, 80972i32, // balled up
    80950i32, 80952i32, // spread out
};

#[derive(Default, Debug, Clone, PartialEq)]
enum ModStatus {
    /// Normal controls
    Normal,
    /// QWOP controls
    #[default]
    Qwop,
    /// Normal pose but don't allow moving. This is temporarily switched on during animations
    /// where the player is sitting down or performing another special action that prevents walking.
    Frozen,
}

#[derive(Default)]
pub struct QwopMod {
    input_state: QwopInputState,
    physics: QwopPhysics,
    prev_main_player_loaded: bool,
    displacement: Vec3,
    status: ModStatus,
    transition_progress: f32,
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
            self.transition_progress = 0.0;
        }

        self.input_state.poll();

        let slow_transition = player
            .modules
            .time_act
            .anim_queue
            .iter()
            .any(|anim| SLOW_TRANSITION_ANIMATIONS.contains(&anim.anim_id));

        if self.input_state.disabled {
            self.status = ModStatus::Normal;
        } else if
        // Disable QWOP while performing a crit or being critted
        player.modules.throw.throw_node.throw_state != ThrowNodeState::None
        // Disable QWOP during certain animations
        || player.modules.time_act.anim_queue.iter().any(|anim| EXCLUDED_ANIMATIONS.contains(&anim.anim_id))
        {
            self.status = ModStatus::Frozen;
        } else {
            self.status = ModStatus::Qwop;
        }

        if self.status == ModStatus::Qwop {
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
                    fe_man.frontend_values.full_screen_message_request_id = 38;
                    self.cheese_discovered = true;
                }
            }
        }

        let mut transition_speed = if slow_transition { 0.5 } else { 3.0 };
        if self.status != ModStatus::Qwop {
            transition_speed *= -1.0;
        }
        self.transition_progress += transition_speed * player.modules.hitstop.frame_time;
        self.transition_progress = self.transition_progress.max(0.0).min(1.0);

        let disable_normal_movement = self.status != ModStatus::Normal;

        // No normal walking, sneaking, rolling, backstep, or horse allowed while QWOP is enabled.
        let disabled_actions = &mut player.modules.action_request.disabled_action_inputs;
        disabled_actions.set_l3(disable_normal_movement);
        disabled_actions.set_rolling(disable_normal_movement);
        disabled_actions.set_backstep(disable_normal_movement);
        player
            .debug_flags
            .set_disabled_movement(disable_normal_movement);

        if let Some(horse_whistle) = unsafe { SoloParamRepository::instance_mut() }
            .ok()
            .and_then(|solo_param_repository| {
                solo_param_repository.get_mut::<EquipParamGoods>(SPECTRAL_STEED_WHISTLE_GOODS_ID)
            })
        {
            horse_whistle.set_enable_live(!disable_normal_movement)
        }
    }

    /// Update the player's root motion based on the current QWOP physics state. This must be done
    /// in a hook in the middle of the HavokBehavior task group, after the player's root motion has
    /// been set but before it is applied.
    pub fn chr_ctrl_update_pos_hook(&mut self, chr_ctrl: NonNull<ChrCtrl>) {
        if self.status != ModStatus::Qwop {
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
        let disable_turning = player
            .modules
            .action_flag
            .action_modifiers_flags
            .disable_turning();

        if !disable_turning && let Ok(cs_camera) = unsafe { CSCamera::instance() } {
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
        if self.status != ModStatus::Normal
            && (self.physics.fallen()
                || unsafe { player.player_game_data.as_ref() }.current_hp == 0)
        {
            player.set_ragdoll(true);
            return;
        }

        let lerp_amount = 1.0 - f32::cos(self.transition_progress * f32::consts::PI / 2.0);

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
