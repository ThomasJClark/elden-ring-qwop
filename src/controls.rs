use eldenring::{
    cs::{CSKeyboardKey, CSPcKeyConfig, KeyAssignID, UserInputKey},
    fd4::FD4PadManager,
};
use fromsoftware_shared::{FromStatic, Program};
use pelite::pe::Pe;

use crate::rvas;

const VANILLA_CONTROLS: [(KeyAssignID, CSKeyboardKey); 4] = [
    (KeyAssignID::MoveForwards, CSKeyboardKey::W),
    (KeyAssignID::MoveLeft, CSKeyboardKey::A),
    (KeyAssignID::MoveBackwards, CSKeyboardKey::S),
    (KeyAssignID::MoveRight, CSKeyboardKey::D),
];

const CONTROLS: [(KeyAssignID, CSKeyboardKey); 9] = [
    (KeyAssignID::MovementControl, CSKeyboardKey::K),
    (KeyAssignID::MoveForwards, CSKeyboardKey::Q),
    (KeyAssignID::MoveBackwards, CSKeyboardKey::W),
    (KeyAssignID::MoveLeft, CSKeyboardKey::O),
    (KeyAssignID::MoveRight, CSKeyboardKey::P),
    (KeyAssignID::CrouchStandUp, CSKeyboardKey::None),
    (KeyAssignID::BackstepDodgeRollDash, CSKeyboardKey::None),
    (KeyAssignID::Jump, CSKeyboardKey::None),
    (
        KeyAssignID::ResetCameraLockOnRemoveTarget,
        CSKeyboardKey::None,
    ),
];

pub struct QwopControls {
    pub q: bool,
    pub w: bool,
    pub o: bool,
    pub p: bool,
    pub disabled: bool,
    prev_disable_input: bool,
}

impl QwopControls {
    pub fn new() -> Self {
        Self {
            q: false,
            w: false,
            o: false,
            p: false,
            disabled: false,
            prev_disable_input: false,
        }
    }

    pub fn poll(&mut self) {
        // When the mod is first loaded, change the default controls from WASD to QWOP. The FD4Pad
        // system is used for input instead of raw windows input so players can edit keybindings
        // in the game settings, but we have to make the initial controls match the QWOP ones
        if let Ok(key_config) = (unsafe { CSPcKeyConfig::instance_mut() })
            && VANILLA_CONTROLS
                .iter()
                .all(|(key_assign_id, keyboard_key_id)| {
                    let key_assign = key_config.key_assign(*key_assign_id);
                    key_assign.keyboard_key_id == *keyboard_key_id
                        && key_assign.keyboard_modify_key.is_none()
                })
        {
            for (key_assign_id, keyboard_key_id) in CONTROLS {
                let key_assign = key_config.key_assign_mut(key_assign_id);
                key_assign.keyboard_key_id = keyboard_key_id;
                key_assign.keyboard_modify_key = None;
            }

            let refresh_user_input_mapping_va = Program::current()
                .rva_to_va(rvas::REFRESH_USER_INPUT_MAPPING)
                .unwrap();
            let refresh_user_input_mapping = unsafe {
                std::mem::transmute::<u64, extern "C" fn() -> ()>(refresh_user_input_mapping_va)
            };
            refresh_user_input_mapping();
        }

        if let Ok(pad_manager) = (unsafe { FD4PadManager::instance() })
            && let Some(pad) = pad_manager.get_in_game_pad()
        {
            self.q = pad.poll_digital_input(UserInputKey::MoveForwards);
            self.w = pad.poll_digital_input(UserInputKey::MoveBackwards);
            self.o = pad.poll_digital_input(UserInputKey::MoveLeft);
            self.p = pad.poll_digital_input(UserInputKey::MoveRight);

            let disable_input = pad.poll_digital_input(UserInputKey::MovementControl);
            if disable_input && !self.prev_disable_input {
                self.disabled = !self.disabled;
            }
            self.prev_disable_input = disable_input;
        };
    }
}
