mod input_state;
mod messages;
mod param;
mod physics;
mod player_ins_skeleton_ext;
mod qwop_mod;
mod rvas;

use std::{
    fs::OpenOptions,
    os::windows::io::AsRawHandle,
    ptr::NonNull,
    sync::{Mutex, OnceLock},
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
    param::SP_EFFECT_PARAM_ST,
};
use fromsoftware_shared::{Program, SharedTaskImpExt};

use pelite::pe64::{Pe, Rva};

use crate::param::{BASE_SP_EFFECT_ID, FALLEN_SP_EFFECT_ID, SpEffectParamLookupResult};
use crate::qwop_mod::QwopMod;
use crate::{
    messages::{MessageCategory, QwopMessages, StaticUtf16String},
    param::get_fallen_sp_effect_param,
};

static FALLEN_SP_EFFECT_PARAM: OnceLock<SP_EFFECT_PARAM_ST> = OnceLock::new();

retour::static_detour! {
    static MsgRepository_LookupEntry: extern "C" fn(NonNull<()>, i32, MessageCategory, i32) -> StaticUtf16String;
    static GetSpEffectParam: extern "C" fn(NonNull<SpEffectParamLookupResult>, i32) -> NonNull<SpEffectParamLookupResult>;
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

    let qwop_mod = Box::leak(Box::new(Mutex::new(QwopMod::default())));

    std::thread::spawn(move || {
        let cs_task = CSTaskImp::wait_for_instance(Duration::MAX).unwrap();
        cs_task.run_recurring(
            |_: &FD4TaskData| qwop_mod.lock().unwrap().chr_ins_pre_behavior(),
            CSTaskGroupIndex::ChrIns_PreBehavior,
        );

        let rva_to_ptr = |rva: Rva| Program::current().rva_to_va(rva).unwrap() as *const ();

        unsafe {
            MsgRepository_LookupEntry
                .initialize(
                    retour::Function::from_ptr(rva_to_ptr(rvas::MSG_REPOSITORY_LOOKUP_ENTRY)),
                    |this, version, category, id| {
                        if let Some(result) = QwopMessages::lookup_message(category, id) {
                            return result;
                        }

                        MsgRepository_LookupEntry.call(this, version, category, id)
                    },
                )
                .unwrap()
                .enable()
                .unwrap();

            GetSpEffectParam
                .initialize(
                    retour::Function::from_ptr(rva_to_ptr(rvas::GET_SP_EFFECT_PARAM)),
                    |mut result, id| {
                        if id == FALLEN_SP_EFFECT_ID {
                            let param =
                                FALLEN_SP_EFFECT_PARAM.get_or_init(|| -> SP_EFFECT_PARAM_ST {
                                    GetSpEffectParam.call(result, BASE_SP_EFFECT_ID);
                                    println!("{:?}", result.as_ref().param);
                                    get_fallen_sp_effect_param(
                                        result.as_ref().param.unwrap().read(),
                                    )
                                });

                            *result.as_mut() = SpEffectParamLookupResult {
                                param: Some(NonNull::from_ref(param)),
                                _id: id,
                                _unkc: 4,
                            };
                        } else {
                            GetSpEffectParam.call(result, id);
                        }

                        result
                    },
                )
                .unwrap()
                .enable()
                .unwrap();

            ChrCtrl_UpdatePos
                .initialize(
                    retour::Function::from_ptr(rva_to_ptr(rvas::CHR_CTRL_UPDATE_POS)),
                    |chr_ctrl| {
                        qwop_mod.lock().unwrap().chr_ctrl_update_pos_hook(chr_ctrl);
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
                        qwop_mod
                            .lock()
                            .unwrap()
                            .chr_ins_behavior_safe_hook(world_chr_man);

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
