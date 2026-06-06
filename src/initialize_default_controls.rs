use eldenring::cs::{CSKeyboardKey, CSPcKeyConfig, KeyAssignID};
use fromsoftware_shared::{FromStatic, Program};
use pelite::pe::Pe;

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

pub fn update_inputs() {
    let key_config = unsafe { CSPcKeyConfig::instance_mut() }.unwrap();

    // When the mod is first loaded, change the default controls from WASD to QWOP. The FD4Pad
    // system is used for input instead of raw windows input so players can edit keybindings
    // in the game settings, but we have to make the initial controls match the QWOP ones
    if VANILLA_CONTROLS
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

        let refresh_user_input_mapping_va = Program::current().rva_to_va(0x243200).unwrap();
        let refresh_user_input_mapping = unsafe {
            std::mem::transmute::<u64, extern "C" fn() -> ()>(refresh_user_input_mapping_va)
        };
        refresh_user_input_mapping();
    }
}
