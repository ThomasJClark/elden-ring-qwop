use core::f32;
use glam::vec4;
use std::ptr::NonNull;

use eldenring::{
    cs::{CSCamera, ChrCtrl, ChrInsExt, EquipParamGoods, SoloParamRepository, WorldChrMan},
    havok::{HkQuaternion, HkVector4},
};
use fromsoftware_shared::FromStatic;

use crate::input_state::QwopInputState;
use crate::physics::QwopPhysics;
use crate::player_ins_skeleton_ext::PlayerInsSkeletonExt;

/// SpEffectParam that applies damage and blood splatter VFX after falling
const FALLEN_SP_EFFECT_ID: i32 = 67;

/// EquipParamGoods for the horse summon wistle. Horse is banned because it trivializes movement
const SPECTRAL_STEED_WHISTLE_GOODS_ID: u32 = 130;

const DAMPING: f32 = 3.0;

#[derive(Default)]
pub struct QwopMod {
    input_state: QwopInputState,
    physics: QwopPhysics,
    prev_main_player_loaded: bool,
    displacement: HkVector4,
}

unsafe impl Sync for QwopMod {}

/// Top level mod logic for the QWOP mod. This is just responsible for gluing together the QWOP
/// state and the Elden Ring state at various hooks and tasks.
impl QwopMod {
    /// The main update loop for the QWOP mod. Polls for input, advances the QWOP physics
    /// simulation, and updates any game state that doesn't need to be a in a specific hook or task
    /// group to avoid getting overwritten.
    pub fn chr_ins_pre_behavior(&mut self) {
        let Some(player) = unsafe { WorldChrMan::instance_mut() }
            .ok()
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
            self.displacement = HkVector4::ZERO;
        }

        self.input_state.poll();

        let qwop_enabled = !self.input_state.disabled;

        if qwop_enabled {
            self.physics.control(
                self.input_state.q,
                self.input_state.w,
                self.input_state.o,
                self.input_state.p,
            );

            self.physics.step(player.modules.hitstop.frame_time);

            // Apply damage and reset the fallen flag when the player falls
            if self.physics.just_fallen() {
                player.apply_speffect(FALLEN_SP_EFFECT_ID, true);
            }
        }

        // No normal walking or horse allowed while QWOP is enabled.
        // TODO: disable evasion actions (roll, sneak, backstep) as well. Currently these are
        // controlled in HKS, which isn't aware of the mod's enabled/disabled status
        player.debug_flags.set_disabled_movement(qwop_enabled);

        if let Some(horse_whistle) = unsafe { SoloParamRepository::instance_mut() }
            .ok()
            .and_then(|solo_param_repository| {
                solo_param_repository.get_mut::<EquipParamGoods>(SPECTRAL_STEED_WHISTLE_GOODS_ID)
            })
        {
            horse_whistle.set_enable_live(!qwop_enabled)
        }
    }

    /// Update the player's root motion based on the current QWOP physics state. This must be done
    /// in a hook in the middle of the HavokBehavior task group, after the player's root motion has
    /// been set but before it is applied.
    pub fn chr_ctrl_update_pos_hook(&mut self, chr_ctrl: NonNull<ChrCtrl>) {
        if let Ok(world_chr_man) = unsafe { WorldChrMan::instance_mut() }
            && let Some(player) = &mut world_chr_man.main_player
            && player.as_ptr() as *const _ == unsafe { chr_ctrl.as_ref() }.owner.as_ptr()
        {
            if self.input_state.disabled {
                return;
            }

            let velocity = self.physics.velocity();
            let frame_time = player.modules.hitstop.frame_time;

            // Allow root motion from animation and physics, but keep track of the total
            // displacement and damp it over time. This ensures players can't cheese movement
            // in the long term by attacking, but attacks don't lose out on short term reach.
            player.modules.behavior.root_motion -= DAMPING * self.displacement * frame_time;
            self.displacement += player.modules.behavior.root_motion;
            self.displacement.w = 0.0; // Gravity and jumping are OK

            // Note that root_motion is already in player coordinates
            player.modules.behavior.root_motion += vec4(0.0, 0.0, -velocity, 0.0) * frame_time;

            // While the player is in motion, face right relative to the camera like in QWOP. There is
            // no way to turn in three dimensions in QWOP, so we can make do by turning the camera
            let snap_rotation = self.input_state.q
                || self.input_state.w
                || self.input_state.o
                || self.input_state.p;

            if snap_rotation && let Ok(cs_camera) = unsafe { CSCamera::instance() } {
                let camera_matrix = cs_camera.pers_cam_2.matrix;
                let camera_angle = f32::atan2(camera_matrix.0.2, camera_matrix.2.2);
                player.modules.physics.orientation = HkQuaternion::from_rotation_y(
                    -(camera_angle + (90.0 + 60.0) * (f32::consts::PI / 180.0)),
                );
            }
        }
    }

    /// Update the main player's skeleton pose based on the current QWOP physics state
    pub fn chr_ins_behavior_safe_hook(&self, mut world_chr_man: NonNull<WorldChrMan>) {
        let Some(player) = &mut unsafe { world_chr_man.as_mut() }.main_player else {
            return;
        };

        player.set_ragdoll(false);

        if self.input_state.disabled {
            return;
        }

        // When the player falls per QWOP rules or dies in real life, ragdoll because it's funny
        if self.physics.fallen() || unsafe { player.player_game_data.as_ref() }.current_hp == 0 {
            player.set_ragdoll(true);
            return;
        }

        player.set_pose(
            self.physics.elevation(),
            self.physics.root_angle(),
            self.physics.neck_angle(),
            self.physics.left_hip_angle(),
            self.physics.right_hip_angle(),
            self.physics.left_knee_angle(),
            self.physics.right_knee_angle(),
            self.physics.left_foot_angle(),
            self.physics.right_foot_angle(),
        );
    }
}
