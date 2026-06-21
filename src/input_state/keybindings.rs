use eldenring::cs::{CSKeyboardKey, CSPcKeyConfig, KeyAssignID};
use fromsoftware_shared::{FromStatic, Program};
use pelite::pe::Pe;

use crate::rvas;

/// Configuration for keyboard controls relevant to QWOP. The keybindings are updated when
/// the mod is loaded, and toggled between vanilla and QWOP when QWOP mode is turned on and off
pub struct Keybindings {
    move_forwards: CSKeyboardKey,
    move_backwards: CSKeyboardKey,
    move_left: CSKeyboardKey,
    move_right: CSKeyboardKey,
    crouch_stand_up: CSKeyboardKey,
    backstep_dodge_roll_dash: CSKeyboardKey,
    jump: CSKeyboardKey,
    reset_camera_lock_on_remove_target: CSKeyboardKey,
}

impl Default for Keybindings {
    /// The iconic default keybindings (hopefully to be changed by the player though)
    fn default() -> Self {
        Self {
            move_forwards: CSKeyboardKey::Q,
            move_backwards: CSKeyboardKey::W,
            move_left: CSKeyboardKey::O,
            move_right: CSKeyboardKey::P,
            crouch_stand_up: CSKeyboardKey::None,
            backstep_dodge_roll_dash: CSKeyboardKey::None,
            jump: CSKeyboardKey::None,
            reset_camera_lock_on_remove_target: CSKeyboardKey::None,
        }
    }
}

impl Keybindings {
    /// Returns the current keybinding settings per [CSPcKeyConfig]
    pub fn current() -> Self {
        let Ok(key_config) = (unsafe { CSPcKeyConfig::instance() }) else {
            return Default::default();
        };

        let get_key = |key_assign_id: KeyAssignID| {
            let key_assign = key_config.key_assign(key_assign_id);
            key_assign.keyboard_key_id
        };

        Self {
            move_forwards: get_key(KeyAssignID::MoveForwards),
            move_backwards: get_key(KeyAssignID::MoveBackwards),
            move_left: get_key(KeyAssignID::MoveLeft),
            move_right: get_key(KeyAssignID::MoveRight),
            crouch_stand_up: get_key(KeyAssignID::CrouchStandUp),
            backstep_dodge_roll_dash: get_key(KeyAssignID::BackstepDodgeRollDash),
            jump: get_key(KeyAssignID::Jump),
            reset_camera_lock_on_remove_target: get_key(KeyAssignID::ResetCameraLockOnRemoveTarget),
        }
    }

    /// Returns the default keybindings for the vanilla game
    pub fn vanilla() -> Self {
        Self {
            move_forwards: CSKeyboardKey::W,
            move_backwards: CSKeyboardKey::S,
            move_left: CSKeyboardKey::A,
            move_right: CSKeyboardKey::D,
            crouch_stand_up: CSKeyboardKey::C,
            backstep_dodge_roll_dash: CSKeyboardKey::LeftShift,
            jump: CSKeyboardKey::Space,
            reset_camera_lock_on_remove_target: CSKeyboardKey::Q,
        }
    }

    /// Returns true if the keybindings are currently set to the default WASD controls. This is
    /// used as a heuristic to determine if QWOP controls were applied
    pub fn is_wasd(&self) -> bool {
        self.move_forwards == CSKeyboardKey::W
            && self.move_left == CSKeyboardKey::A
            && self.move_backwards == CSKeyboardKey::S
            && self.move_right == CSKeyboardKey::D
    }

    /// Applies the keybinding settings to the game. They can then be polled on the next frame.
    pub fn apply(&self) {
        let Ok(key_config) = (unsafe { CSPcKeyConfig::instance_mut() }) else {
            return;
        };

        let mut assign_key = |key_assign_id: KeyAssignID, keyboard_key_id: CSKeyboardKey| {
            let key_assign = key_config.key_assign_mut(key_assign_id);
            key_assign.keyboard_key_id = keyboard_key_id;
            key_assign.keyboard_modify_key = None;
        };

        assign_key(KeyAssignID::MoveForwards, self.move_forwards);
        assign_key(KeyAssignID::MoveLeft, self.move_left);
        assign_key(KeyAssignID::MoveBackwards, self.move_backwards);
        assign_key(KeyAssignID::MoveRight, self.move_right);
        assign_key(KeyAssignID::CrouchStandUp, self.crouch_stand_up);
        assign_key(
            KeyAssignID::BackstepDodgeRollDash,
            self.backstep_dodge_roll_dash,
        );
        assign_key(KeyAssignID::Jump, self.jump);
        assign_key(
            KeyAssignID::ResetCameraLockOnRemoveTarget,
            self.reset_camera_lock_on_remove_target,
        );

        let refresh_user_input_mapping = unsafe {
            std::mem::transmute::<u64, extern "C" fn() -> ()>(
                Program::current()
                    .rva_to_va(rvas::REFRESH_USER_INPUT_MAPPING)
                    .unwrap(),
            )
        };
        refresh_user_input_mapping();
    }
}
