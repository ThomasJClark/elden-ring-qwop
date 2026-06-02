mod qwop_control;
mod qwop_physics;

use glam::vec4;
use std::{
    f32::consts::PI,
    fs::OpenOptions,
    os::windows::io::AsRawHandle,
    ptr::NonNull,
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use windows::Win32::{
    Foundation::{HANDLE, HINSTANCE},
    System::{
        Console::{AllocConsole, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle},
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::DLL_PROCESS_ATTACH,
    },
    UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_F6, VK_F7},
};

use eldenring::{
    cs::{ChrCtrl, PlayerIns, UserInputKey, WorldChrMan},
    fd4::FD4PadManager,
    havok::HkQuaternion,
    util::system::wait_for_system_init,
};
use fromsoftware_shared::{FromStatic, Program};
use pelite::pe64::Pe;
use retour::static_detour;

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

static QWOP: LazyLock<Mutex<qwop_control::QwopControl>> =
    LazyLock::new(|| Mutex::new(qwop_control::QwopControl::new()));

static DISABLE_QWOP_CONTROL: AtomicBool = AtomicBool::new(false);

static_detour! {
    static ChrIns_PreBehaviorSafe: extern "C" fn(NonNull<WorldChrMan>);
    static ChrCtrl_UpdatePos: extern "C" fn(NonNull<ChrCtrl>);
}

/// Advances the QWOP simulation based on the current inputs, and updates the character's pose.
/// This is called in a hook just before the player's pose is read by the game, so we can override
/// certain bones with our own pose
fn chr_ins_pre_behavior_safe_detour(player: &mut PlayerIns) {
    let Ok(mut qwop) = QWOP.lock() else {
        return;
    };
    let Ok(pad_manager) = (unsafe { FD4PadManager::instance() }) else {
        return;
    };
    let Some(pad) = pad_manager.get_in_game_pad() else {
        return;
    };

    // Temporary for testing. F7 = reset, F6 = toggle normal controls
    if unsafe { GetAsyncKeyState(VK_F7.0.into()) } & 1 != 0 {
        player.chr_ctrl.chr_ragdoll_state = 0;
        qwop.reset();
    }
    if unsafe { GetAsyncKeyState(VK_F6.0.into()) } & 1 != 0 {
        DISABLE_QWOP_CONTROL.fetch_xor(true, Ordering::Relaxed);
    }
    if DISABLE_QWOP_CONTROL.load(Ordering::Relaxed) {
        player.chr_ctrl.chr_ragdoll_state = 0;
        qwop.reset();
        return;
    }

    // The movement keys are remapped to QWOP controls, since afaik there isn't a good way
    // to add completely new controls, and the player can't do normal WASD movement anyway
    qwop.control(
        pad.poll_digital_input(UserInputKey::MoveForwards), // Q
        pad.poll_digital_input(UserInputKey::MoveBackwards), // W
        pad.poll_digital_input(UserInputKey::MoveLeft),     // O
        pad.poll_digital_input(UserInputKey::MoveRight),    // P
    );

    qwop.step(player.modules.hitstop.frame_time);

    // When the player falls per QWOP rules, damage them and ragdoll. In real QWOP, this just
    // immediately restarts the game.
    if qwop.fallen {
        player.chr_ctrl.chr_ragdoll_state = 2;
        return;
    } else {
        player.chr_ctrl.chr_ragdoll_state = 0;
    }

    // Also when the player dies, ragdoll and stop updating the pose
    if unsafe { player.player_game_data.as_ref() }.current_hp == 0 {
        player.chr_ctrl.chr_ragdoll_state = 2;
        return;
    }

    let Some(havok_context) = &mut player.modules.behavior.havok_context else {
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
                    reference_poses[bone_index].translation.y + qwop.physics.elevation();
                pose.rotation * HkQuaternion::from_rotation_x(qwop.physics.root_angle())
            }
            "Neck" => HkQuaternion::from_rotation_y(qwop.physics.neck_angle()),
            "L_Thigh" => HkQuaternion::from_euler(
                glam::EulerRot::XZY,
                0.0,
                -PI,
                qwop.physics.left_hip_angle(),
            ),
            "R_Thigh" => HkQuaternion::from_euler(
                glam::EulerRot::XZY,
                0.0,
                -PI,
                qwop.physics.right_hip_angle(),
            ),
            "L_Calf" => HkQuaternion::from_rotation_y(qwop.physics.left_knee_angle()),
            "R_Calf" => HkQuaternion::from_rotation_y(qwop.physics.right_knee_angle()),
            "L_Foot" => HkQuaternion::from_rotation_y(qwop.physics.left_foot_angle()),
            "R_Foot" => HkQuaternion::from_rotation_y(qwop.physics.right_foot_angle()),
            _ => pose.rotation,
        };
    }
}

/// Update the player's root motion based on the velocity in the QWOP simulation. This is called
/// in a separate hook right after the root motion vector is overritten each physics step
fn chr_ctrl_update_pos_detour(player: &mut PlayerIns) {
    if DISABLE_QWOP_CONTROL.load(Ordering::Relaxed) {
        player.debug_flags.set_disabled_movement(false);
        return;
    }

    let Ok(qwop) = QWOP.lock() else {
        return;
    };

    let motion = -qwop.physics.velocity() * player.modules.hitstop.frame_time;
    player.modules.behavior.root_motion = vec4(0.0, 0.0, motion, 0.0);
    player.debug_flags.set_disabled_movement(true);
}

#[unsafe(no_mangle)]
pub extern "C" fn DllMain(module: HINSTANCE, reason: u32) -> bool {
    if reason != DLL_PROCESS_ATTACH {
        return true;
    }

    unsafe { DisableThreadLibraryCalls(module.into()) }.unwrap();

    unsafe {
        AllocConsole().unwrap();
        let stdout = OpenOptions::new().write(true).open("CONOUT$").unwrap();
        let stderr = OpenOptions::new().write(true).open("CONOUT$").unwrap();
        SetStdHandle(STD_OUTPUT_HANDLE, HANDLE(stdout.as_raw_handle() as _)).unwrap();
        SetStdHandle(STD_ERROR_HANDLE, HANDLE(stderr.as_raw_handle() as _)).unwrap();
        std::mem::forget(stdout);
        std::mem::forget(stderr);
    };

    std::thread::spawn(move || {
        wait_for_system_init(&Program::current(), Duration::MAX).unwrap();

        unsafe {
            ChrIns_PreBehaviorSafe
                .initialize(
                    std::mem::transmute::<u64, extern "C" fn(NonNull<WorldChrMan>)>(
                        Program::current().rva_to_va(0x50fe10).unwrap(),
                    ),
                    move |mut world_chr_man: NonNull<WorldChrMan>| {
                        if let Some(main_player) = &mut world_chr_man.as_mut().main_player {
                            chr_ins_pre_behavior_safe_detour(main_player);
                        }

                        ChrIns_PreBehaviorSafe.call(world_chr_man);
                    },
                )
                .unwrap()
                .enable()
                .unwrap();

            ChrCtrl_UpdatePos
                .initialize(
                    std::mem::transmute::<u64, extern "C" fn(NonNull<ChrCtrl>)>(
                        Program::current().rva_to_va(0x3c8610).unwrap(),
                    ),
                    |chr_ctrl: NonNull<ChrCtrl>| {
                        if let Ok(world_chr_man) = WorldChrMan::instance_mut()
                            && let Some(main_player) = &mut world_chr_man.main_player
                            && main_player.as_ptr() as *const _ == chr_ctrl.as_ref().owner.as_ptr()
                        {
                            chr_ctrl_update_pos_detour(main_player);
                        }

                        ChrCtrl_UpdatePos.call(chr_ctrl);
                    },
                )
                .unwrap()
                .enable()
                .unwrap();
        }
    });

    true
}
