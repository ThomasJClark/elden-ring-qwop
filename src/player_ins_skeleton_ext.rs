use std::f32::consts::PI;

use eldenring::{cs::PlayerIns, havok::HkQuaternion};

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

pub trait PlayerInsSkeletonExt {
    fn set_ragdoll(&mut self, ragdoll: bool);

    #[allow(clippy::too_many_arguments)]
    fn set_pose(
        &mut self,
        elevation: f32,
        root_angle: f32,
        neck_angle: f32,
        left_hip_angle: f32,
        right_hip_angle: f32,
        left_knee_angle: f32,
        right_knee_angle: f32,
        left_foot_angle: f32,
        right_foot_angle: f32,
    );
}

impl PlayerInsSkeletonExt for PlayerIns {
    fn set_ragdoll(&mut self, ragdoll: bool) {
        self.chr_ctrl.chr_ragdoll_state = if ragdoll { 2 } else { 0 };
    }

    /// Applies the QWOP physics state to the player character's skeleton pose
    fn set_pose(
        &mut self,
        elevation: f32,
        root_angle: f32,
        neck_angle: f32,
        left_hip_angle: f32,
        right_hip_angle: f32,
        left_knee_angle: f32,
        right_knee_angle: f32,
        left_foot_angle: f32,
        right_foot_angle: f32,
    ) {
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
                    pose.translation.y = reference_poses[bone_index].translation.y + elevation;
                    pose.rotation * HkQuaternion::from_rotation_x(root_angle)
                }
                "Neck" => HkQuaternion::from_rotation_y(neck_angle),
                "L_Thigh" => {
                    HkQuaternion::from_euler(glam::EulerRot::XZY, 0.0, -PI, left_hip_angle)
                }
                "R_Thigh" => {
                    HkQuaternion::from_euler(glam::EulerRot::XZY, 0.0, -PI, right_hip_angle)
                }
                "L_Calf" => HkQuaternion::from_rotation_y(left_knee_angle),
                "R_Calf" => HkQuaternion::from_rotation_y(right_knee_angle),
                "L_Foot" => HkQuaternion::from_rotation_y(left_foot_angle),
                "R_Foot" => HkQuaternion::from_rotation_y(right_foot_angle),
                _ => pose.rotation,
            };
        }
    }
}
