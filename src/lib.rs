#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]

mod input_state;
mod physics;
mod player_ins_skeleton_ext;
mod qwop_mod;
mod rvas;

use std::{
    cell::SyncUnsafeCell, fs::OpenOptions, os::windows::io::AsRawHandle, ptr::NonNull,
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
    cs::{CSTaskGroupIndex, CSTaskImp, ChrCtrl, WorldChrMan},
    fd4::FD4TaskData,
};
use fromsoftware_shared::{Program, SharedTaskImpExt};
use pelite::pe64::{Pe, Rva};

use crate::qwop_mod::QwopMod;

static QWOP_MOD: SyncUnsafeCell<Option<QwopMod>> = SyncUnsafeCell::new(None);

fn qwop_mod_instance() -> &'static mut QwopMod {
    unsafe { QWOP_MOD.get().as_mut_unchecked() }.get_or_insert_default()
}

retour::static_detour! {
    static ChrCtrl_UpdatePos: extern "C" fn(NonNull<ChrCtrl>);
    static ChrIns_BehaviorSafe: extern "C" fn(NonNull<WorldChrMan>);
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
        SetStdHandle(STD_OUTPUT_HANDLE, HANDLE(stdout.as_raw_handle() as _)).unwrap();
        SetStdHandle(STD_ERROR_HANDLE, HANDLE(stdout.as_raw_handle() as _)).unwrap();
        std::mem::forget(stdout);
    };

    std::thread::spawn(move || {
        let cs_task = CSTaskImp::wait_for_instance(Duration::MAX).unwrap();
        cs_task.run_recurring(
            |_: &FD4TaskData| qwop_mod_instance().chr_ins_pre_behavior(),
            CSTaskGroupIndex::ChrIns_PreBehavior,
        );

        let rva_to_ptr = |rva: Rva| Program::current().rva_to_va(rva).unwrap() as *const ();

        unsafe {
            ChrCtrl_UpdatePos
                .initialize(
                    retour::Function::from_ptr(rva_to_ptr(rvas::CHR_CTRL_UPDATE_POS)),
                    |chr_ctrl| {
                        qwop_mod_instance().chr_ctrl_update_pos_hook(chr_ctrl);
                        ChrCtrl_UpdatePos.call(chr_ctrl);
                    },
                )
                .unwrap()
                .enable()
                .unwrap();

            ChrIns_BehaviorSafe
                .initialize(
                    retour::Function::from_ptr(rva_to_ptr(rvas::CHR_INS_BEHAVIOR_SAFE)),
                    |world_chr_man| {
                        qwop_mod_instance().chr_ins_behavior_safe_hook(world_chr_man);
                        ChrIns_BehaviorSafe.call(world_chr_man);
                    },
                )
                .unwrap()
                .enable()
                .unwrap();
        }
    });

    true
}
