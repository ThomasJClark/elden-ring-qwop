mod controls;
mod physics;
mod rvas;
mod skeleton_sync;

use glam::vec4;
use std::{
    fs::OpenOptions,
    os::windows::io::AsRawHandle,
    ptr::NonNull,
    sync::{LazyLock, Mutex},
    time::Duration,
};
use windows::Win32::{
    Foundation::{HANDLE, HINSTANCE},
    System::{
        Console::{AllocConsole, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE, SetStdHandle},
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::DLL_PROCESS_ATTACH,
    },
};

use eldenring::{
    cs::{CSTaskGroupIndex, CSTaskImp, ChrCtrl, PlayerIns, WorldChrMan},
    fd4::FD4TaskData,
    util::system::wait_for_system_init,
};
use fromsoftware_shared::{FromStatic, Program, SharedTaskImpExt};
use pelite::pe64::Pe;
use retour::static_detour;

use crate::controls::QwopControls;
use crate::physics::QwopPhysics;

static QWOP_CONTROLS: LazyLock<Mutex<QwopControls>> =
    LazyLock::new(|| Mutex::new(QwopControls::new()));

static QWOP_PHYSICS: LazyLock<Mutex<physics::QwopPhysics>> =
    LazyLock::new(|| Mutex::new(QwopPhysics::new()));

static_detour! {
    static ChrIns_PreBehaviorSafe: extern "C" fn(NonNull<WorldChrMan>);
    static ChrCtrl_UpdatePos: extern "C" fn(NonNull<ChrCtrl>);
}

fn main_update() {
    QWOP_CONTROLS.lock().unwrap().poll();
}

/// Advances the QWOP simulation based on the current inputs, and updates the character's pose.
/// This is called in a hook just before the player's pose is read by the game, so we can override
/// certain bones with our own pose
fn chr_ins_pre_behavior_safe_detour(player: &mut PlayerIns) {
    let Ok(mut qwop_physics) = QWOP_PHYSICS.lock() else {
        return;
    };

    if let Ok(qwop_controls) = QWOP_CONTROLS.lock() {
        if qwop_controls.disabled {
            return;
        }

        qwop_physics.control(
            qwop_controls.q,
            qwop_controls.w,
            qwop_controls.o,
            qwop_controls.p,
        );
    };

    qwop_physics.step(player.modules.hitstop.frame_time);

    skeleton_sync::apply_skeleton(player, &qwop_physics);
}

/// Update the player's root motion based on the velocity in the QWOP simulation. This is called
/// in a separate hook right after the root motion vector is overritten each physics step
fn chr_ctrl_update_pos_detour(player: &mut PlayerIns) {
    if QWOP_CONTROLS
        .lock()
        .is_ok_and(|qwop_controls| qwop_controls.disabled)
    {
        player.debug_flags.set_disabled_movement(false);
        return;
    }

    if let Ok(qwop) = QWOP_PHYSICS.lock() {
        let motion = -qwop.velocity() * player.modules.hitstop.frame_time;
        player.modules.behavior.root_motion = vec4(0.0, 0.0, motion, 0.0);
        player.debug_flags.set_disabled_movement(true);
    };
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
            let cs_task = CSTaskImp::wait_for_instance(Duration::MAX).unwrap();
            cs_task.run_recurring(
                |_: &FD4TaskData| {
                    main_update();
                },
                CSTaskGroupIndex::FrameBegin,
            );

            ChrIns_PreBehaviorSafe
                .initialize(
                    std::mem::transmute::<u64, extern "C" fn(NonNull<WorldChrMan>)>(
                        Program::current()
                            .rva_to_va(rvas::CHR_INS_PRE_BEHAVIOR_SAFE)
                            .unwrap(),
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
                        Program::current()
                            .rva_to_va(rvas::CHR_CTRL_UPDATE_POS)
                            .unwrap(),
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
