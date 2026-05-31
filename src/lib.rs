mod qwop;

use std::{
    f32::consts::PI,
    fs::OpenOptions,
    os::windows::io::AsRawHandle,
    ptr::NonNull,
    sync::{Arc, Mutex},
    time::Duration,
};
use windows::Win32::{
    Foundation::{HANDLE, HINSTANCE},
    System::{
        Console::{AllocConsole, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle},
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::DLL_PROCESS_ATTACH,
    },
    UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_E, VK_F4, VK_O, VK_P, VK_Q, VK_R, VK_W},
};

use eldenring::{
    cs::{ChrCtrl, PlayerIns, WorldChrMan},
    havok::HkQuaternion,
    util::system::wait_for_system_init,
};
use fromsoftware_shared::{FromStatic, Program};
use pelite::pe64::Pe;
use retour::static_detour;

use crate::qwop::SkeletonState;

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

static_detour! {
    static ChrIns_PreBehaviorSafe: extern "C" fn(NonNull<WorldChrMan>);
    static ChrCtrl_UpdatePos: extern "C" fn(NonNull<ChrCtrl>);
}

/// Update method called each physics step. This overrides the player's poses set by the normal
/// Havok animation system
fn update_skeleton(player: &mut PlayerIns, skeleton: SkeletonState) {
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

    for (bone_index, bone) in bones.iter().enumerate() {
        let pose = &mut poses[bone_index];
        pose.rotation = match bone.name.to_str() {
            s if LOCK_ROTATION_BONES.contains(s) => reference_poses[bone_index].rotation,
            "RootPos" => {
                pose.translation.y = reference_poses[bone_index].translation.y + skeleton.elevation;
                pose.rotation * HkQuaternion::from_rotation_x(skeleton.root)
            }
            "L_Thigh" => HkQuaternion::from_euler(glam::EulerRot::XZY, 0.0, -PI, skeleton.left_hip),
            "R_Thigh" => {
                HkQuaternion::from_euler(glam::EulerRot::XZY, 0.0, -PI, skeleton.right_hip)
            }
            "L_Calf" => HkQuaternion::from_rotation_y(skeleton.left_knee),
            "R_Calf" => HkQuaternion::from_rotation_y(skeleton.right_knee),
            "L_Foot" => HkQuaternion::from_rotation_y(skeleton.left_foot),
            "R_Foot" => HkQuaternion::from_rotation_y(skeleton.right_foot),
            _ => pose.rotation,
        };
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn DllMain(module: HINSTANCE, reason: u32) -> bool {
    if reason == DLL_PROCESS_ATTACH {
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

            let qwop = Arc::new(Mutex::new(qwop::Qwop::new()));

            unsafe {
                // WorldChrMan
                ChrIns_PreBehaviorSafe
                    .initialize(
                        {
                            let va = Program::current().rva_to_va(0x50fe10).unwrap();
                            std::mem::transmute::<u64, extern "C" fn(NonNull<WorldChrMan>)>(va)
                        },
                        {
                            let qwop = qwop.clone();
                            move |mut world_chr_man: NonNull<WorldChrMan>| {
                                if let Some(main_player) = &mut world_chr_man.as_mut().main_player {
                                    let mut qwop = qwop.lock().unwrap();

                                    if GetAsyncKeyState(VK_F4.0.into()) & 0x0001 != 0 {
                                        qwop.reset();
                                    }

                                    qwop.control(
                                        GetAsyncKeyState(VK_Q.0.into()) < 0
                                            || GetAsyncKeyState(VK_E.0.into()) < 0,
                                        GetAsyncKeyState(VK_W.0.into()) < 0
                                            || GetAsyncKeyState(VK_R.0.into()) < 0,
                                        GetAsyncKeyState(VK_O.0.into()) < 0,
                                        GetAsyncKeyState(VK_P.0.into()) < 0,
                                    );

                                    qwop.step(main_player.modules.hitstop.frame_time);

                                    update_skeleton(main_player.as_mut(), qwop.skeleton());
                                }
                                ChrIns_PreBehaviorSafe.call(world_chr_man);
                            }
                        },
                    )
                    .unwrap()
                    .enable()
                    .unwrap();

                ChrCtrl_UpdatePos
                    .initialize(
                        {
                            let va = Program::current().rva_to_va(0x3c8610).unwrap();
                            std::mem::transmute::<u64, extern "C" fn(NonNull<ChrCtrl>)>(va)
                        },
                        {
                            let qwop = qwop.clone();
                            move |chr_ctrl: NonNull<ChrCtrl>| {
                                if let Ok(world_chr_man) = WorldChrMan::instance_mut()
                                    && let Some(main_player) = &mut world_chr_man.main_player
                                    && main_player.as_ptr() as *const _
                                        == chr_ctrl.as_ref().owner.as_ptr()
                                {
                                    main_player.debug_flags.set_disabled_movement(true);
                                    main_player.modules.behavior.root_motion.z +=
                                        -qwop.lock().unwrap().velocity()
                                            * main_player.modules.hitstop.frame_time;
                                }

                                ChrCtrl_UpdatePos.call(chr_ctrl);
                            }
                        },
                    )
                    .unwrap()
                    .enable()
                    .unwrap();
            }
        });
    }

    true
}
