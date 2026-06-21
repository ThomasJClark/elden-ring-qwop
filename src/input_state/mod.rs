mod keybindings;

use eldenring::{cs::UserInputKey, fd4::FD4PadManager};
use fromsoftware_shared::FromStatic;
use keybindings::Keybindings;

/// Manages input state for QWOP, and updates the keybindings when QWOP is enabled. Note that
/// the FD4Pad system is used for input instead of raw Windows APIs so that the keybindings can
/// be changed in the game settings, so we need to change the actual game keybinding settings when
/// QWOP is enabled and disabled
#[derive(Default)]
pub struct QwopInputState {
    pub q: bool,
    pub w: bool,
    pub o: bool,
    pub p: bool,
    pub disabled: bool,
    prev_disable_key_pressed: bool,

    /// Prevents input polling until after we set up keybindings. We need to wait 1 frame after
    /// so the FD4Pad state is updated after the keybindings are changed
    initialized_keybindings: bool,

    /// Keybindings to use when QWOP is enabled. A mutable copy of this array is kept so that
    /// QWOP controls can be temporarily disabled and enabled without resetting custom keybinding
    /// preferences.
    keybindings: Keybindings,
}

impl QwopInputState {
    pub fn poll(&mut self) {
        if self.initialized_keybindings
            && let Ok(pad_manager) = (unsafe { FD4PadManager::instance() })
            && let Some(pad) = pad_manager.get_in_game_pad()
        {
            let disable_key_pressed = pad.poll_digital_input(UserInputKey::MovementControl);
            if disable_key_pressed && !self.prev_disable_key_pressed {
                self.disabled = !self.disabled;
            }
            self.prev_disable_key_pressed = disable_key_pressed;

            self.q = !self.disabled && pad.poll_digital_input(UserInputKey::MoveForwards);
            self.w = !self.disabled && pad.poll_digital_input(UserInputKey::MoveBackwards);
            self.o = !self.disabled && pad.poll_digital_input(UserInputKey::MoveLeft);
            self.p = !self.disabled && pad.poll_digital_input(UserInputKey::MoveRight);
        };

        // If the current disabled state doesn't match the current keybindings, either revert the
        // keybindings to the default vanilla settings, or restore the keybindings that were in
        // place before QWOP was toggled off
        let keybindings = Keybindings::current();
        if keybindings.is_wasd() != self.disabled {
            if self.disabled {
                self.keybindings = keybindings;
                Keybindings::vanilla().apply();
            } else {
                self.keybindings.apply();
            }
        }

        self.initialized_keybindings = true;
    }
}
