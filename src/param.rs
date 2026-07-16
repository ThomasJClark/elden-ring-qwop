use std::ptr::NonNull;

use eldenring::param::SP_EFFECT_PARAM_ST;

pub struct SpEffectParamLookupResult {
    pub param: Option<NonNull<SP_EFFECT_PARAM_ST>>,
    pub _id: i32,
    pub _unkc: i32,
}

/// SpEffectParam that applies damage and blood splatter VFX after falling
pub const FALLEN_SP_EFFECT_ID: i32 = 67;

pub const BASE_SP_EFFECT_ID: i32 = 10600;

/// Gets the SpEffectParam applied when the player falls, given a copy of vanilla SpEffectParam
/// 10600
pub fn get_fallen_sp_effect_param(param: SP_EFFECT_PARAM_ST) -> SP_EFFECT_PARAM_ST {
    let mut param = param;
    // Subtrack 1/10 of HP rounded up each time you fall
    param.set_change_hp_point(1);
    param.set_change_hp_rate(10.0);
    // Funny blood spatter animation
    param.set_vfx_id(6);
    param
}
