use std::f32::consts::PI;

use eldenring::{cs::PlayerIns, havok::HkQuaternion};

use crate::physics::QwopPhysics;

/// Bones that are locked into the reference pose at all times
static LOCK_ROTATION_BONES: phf::Set<&'static str> = phf::phf_set! {
    "Pelvis",
    "L_Hip" | "R_Hip",
    "L_Calf_Skirt" | "R_Calf_Skirt",
    "L_CalfTwist" | "R_CalfTwist",
    "L_CalfTwist1" | "R_CalfTwist1",
    "L_Foot_Dummy1" | "R_Foot_Dummy1",
    "L_Foot_Dummy2" | "R_Foot_Dummy2",
    "L_FootTwist" | "R_FootTwist",
    "L_Toe0" | "R_Toe0",
    "L_Knee" | "R_Knee",
    "L_Knee_Skirt" | "R_Knee_Skirt",
    "L_Thigh_Skirt" | "R_Thigh_Skirt",
    "L_ThighTwist" | "R_ThighTwist",
    "L_ThighTwist1" | "R_ThighTwist1",
};

pub trait PlayerInsSkeletonSync {
    fn set_ragdoll(&mut self, ragdoll: bool);

    fn apply_skeleton(&mut self, qwop: &QwopPhysics);
}

impl PlayerInsSkeletonSync for PlayerIns {
    fn set_ragdoll(&mut self, ragdoll: bool) {
        self.chr_ctrl.chr_ragdoll_state = if ragdoll { 2 } else { 0 };
    }

    /// Applies the QWOP physics state to the player character's skeleton pose
    fn apply_skeleton(&mut self, qwop: &QwopPhysics) {
        // When the player falls per QWOP rules or dies in the game, ragdoll because it's funny
        let ragdoll = qwop.fallen() || unsafe { self.player_game_data.as_ref() }.current_hp == 0;
        if ragdoll {
            self.set_ragdoll(true);
            return;
        }

        let Some(havok_context) = &mut self.modules.behavior.havok_context else {
            return;
        };

        let hkb_character = &mut *havok_context.character;
        let bones = &hkb_character.setup.skeleton.bones.as_slice();
        let reference_poses = &hkb_character.setup.skeleton.reference_pose.as_slice();
        let poses = hkb_character.state.poses_mut(bones.len());

        // Update the bone poses to match the QWOP simulation
        for (bone_index, bone) in bones.iter().enumerate() {
            let pose = &mut poses[bone_index];
            pose.rotation = match bone.name.to_str() {
                s if LOCK_ROTATION_BONES.contains(s) => reference_poses[bone_index].rotation,
                "RootPos" => {
                    pose.translation.y =
                        reference_poses[bone_index].translation.y + qwop.elevation();
                    pose.rotation * HkQuaternion::from_rotation_x(qwop.root_angle())
                }
                "Neck" => HkQuaternion::from_rotation_y(qwop.neck_angle()),
                "L_Thigh" => {
                    HkQuaternion::from_euler(glam::EulerRot::XZY, 0.0, -PI, qwop.left_hip_angle())
                }
                "R_Thigh" => {
                    HkQuaternion::from_euler(glam::EulerRot::XZY, 0.0, -PI, qwop.right_hip_angle())
                }
                "L_Calf" => HkQuaternion::from_rotation_y(qwop.left_knee_angle()),
                "R_Calf" => HkQuaternion::from_rotation_y(qwop.right_knee_angle()),
                "L_Foot" => HkQuaternion::from_rotation_y(qwop.left_foot_angle()),
                "R_Foot" => HkQuaternion::from_rotation_y(qwop.right_foot_angle()),
                _ => pose.rotation,
            };
        }
    }
}
