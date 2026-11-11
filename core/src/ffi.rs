use crate::audio::{ChimeType, SoundscapePlayer, SoundscapeType};
use crate::dopamine::{self, DopamineAccount, RewardCategory};
use crate::flowmodoro::{FlowState, Flowmodoro};
use crate::storage::{self, TetherSettings};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::time::Duration;

#[derive(Serialize)]
struct FlowSnapshot {
    state: &'static str,
    time: String,
    elapsed_focus_secs: u64,
    remaining_break_secs: u64,
    micro_remaining_secs: u64,
    break_secs_available: u64,
    focus_ratio: u32,
    break_ratio: u32,
    progress: f32,
}

impl From<&Flowmodoro> for FlowSnapshot {
    fn from(flow: &Flowmodoro) -> Self {
        let state = match flow.state {
            FlowState::Idle => "idle",
            FlowState::MicroStart => "micro_start",
            FlowState::Focus => "focus",
            FlowState::Break => "break",
        };
        Self {
            state,
            time: flow.current_time_display(),
            elapsed_focus_secs: flow.elapsed_focus_secs,
            remaining_break_secs: flow.remaining_break_secs,
            micro_remaining_secs: flow.micro_remaining_secs,
            break_secs_available: flow.calculate_break_time(),
            focus_ratio: flow.focus_ratio,
            break_ratio: flow.break_ratio,
            progress: flow.progress_pct(),
        }
    }
}

#[derive(Deserialize)]
struct CoreRequest {
    operation: String,
    #[serde(default)]
    payload: Value,
}

#[derive(Serialize)]
struct CoreResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl CoreResponse {
    fn success(data: Value) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(error: impl Into<String>) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(error.into()),
        }
    }
}

fn dispatch_request(request: CoreRequest) -> Result<Value, String> {
    match request.operation.as_str() {
        "settings.load" => serde_json::to_value(storage::load_settings())
            .map_err(|error| error.to_string()),
        "settings.save" => {
            let settings: TetherSettings =
                serde_json::from_value(request.payload).map_err(|error| error.to_string())?;
            storage::save_settings(&settings).map_err(|error| error.to_string())?;
            Ok(serde_json::json!({ "saved": true }))
        }
        "impulses.list" => {
            let settings = storage::load_settings();
            serde_json::to_value(storage::load_structured_impulses_with_cleanup(&settings))
                .map_err(|error| error.to_string())
        }
        "impulses.add" => {
            #[derive(Deserialize)]
            struct AddImpulse {
                text: String,
            }
            let payload: AddImpulse =
                serde_json::from_value(request.payload).map_err(|error| error.to_string())?;
            let settings = storage::load_settings();
            if !payload.text.trim().is_empty() {
                storage::append_impulse_with_settings(&settings, payload.text.trim())
                    .map_err(|error| error.to_string())?;
            }
            serde_json::to_value(storage::load_structured_impulses_with_cleanup(&settings))
                .map_err(|error| error.to_string())
        }
        "impulses.set_done" => {
            #[derive(Deserialize)]
            struct SetImpulseDone {
                index: usize,
                done: bool,
            }
            let payload: SetImpulseDone =
                serde_json::from_value(request.payload).map_err(|error| error.to_string())?;
            let settings = storage::load_settings();
            storage::toggle_impulse_done(&settings, payload.index, payload.done)
                .map_err(|error| error.to_string())?;
            serde_json::to_value(storage::load_structured_impulses_with_cleanup(&settings))
                .map_err(|error| error.to_string())
        }
        "ramdump.last" => serde_json::to_value(storage::load_last_ramdump())
            .map_err(|error| error.to_string()),
        "ramdump.save" => {
            #[derive(Deserialize)]
            struct SaveRamDump {
                now: String,
                status: String,
                next_step: String,
            }
            let payload: SaveRamDump =
                serde_json::from_value(request.payload).map_err(|error| error.to_string())?;
            if payload.now.trim().is_empty()
                && payload.status.trim().is_empty()
                && payload.next_step.trim().is_empty()
            {
                return Err("Enter at least one RAM-dump field before saving.".to_string());
            }
            storage::append_ramdump(&payload.now, &payload.status, &payload.next_step)
                .map_err(|error| error.to_string())?;
            serde_json::to_value(storage::load_last_ramdump())
                .map_err(|error| error.to_string())
        }
        "dopamine.load" => serde_json::to_value(dopamine::load_dopamine_account())
            .map_err(|error| error.to_string()),
        "dopamine.save" => {
            let account: DopamineAccount =
                serde_json::from_value(request.payload).map_err(|error| error.to_string())?;
            dopamine::save_dopamine_account(&account).map_err(|error| error.to_string())?;
            Ok(serde_json::json!({ "saved": true }))
        }
        "dopamine.redeem" => {
            #[derive(Deserialize)]
            struct RedeemReward {
                reward_id: String,
            }
            let payload: RedeemReward =
                serde_json::from_value(request.payload).map_err(|error| error.to_string())?;
            let mut account = dopamine::load_dopamine_account();
            let redemption = account.redeem(&payload.reward_id)?;
            dopamine::save_dopamine_account(&account).map_err(|error| error.to_string())?;
            serde_json::to_value((account, redemption)).map_err(|error| error.to_string())
        }
        "dopamine.reward.add" => {
            #[derive(Deserialize)]
            struct AddReward {
                title: String,
                cost: u32,
                category: RewardCategory,
            }
            let payload: AddReward =
                serde_json::from_value(request.payload).map_err(|error| error.to_string())?;
            let mut account = dopamine::load_dopamine_account();
            let reward = account.add_custom_reward(payload.title, payload.cost, payload.category);
            dopamine::save_dopamine_account(&account).map_err(|error| error.to_string())?;
            serde_json::to_value((account, reward)).map_err(|error| error.to_string())
        }
        "dopamine.reward.remove" => {
            #[derive(Deserialize)]
            struct RemoveReward {
                reward_id: String,
            }
            let payload: RemoveReward =
                serde_json::from_value(request.payload).map_err(|error| error.to_string())?;
            let mut account = dopamine::load_dopamine_account();
            let removed = account.remove_custom_reward(&payload.reward_id);
            if removed {
                dopamine::save_dopamine_account(&account).map_err(|error| error.to_string())?;
            }
            Ok(serde_json::json!({ "removed": removed }))
        }
        operation => Err(format!("Unsupported core operation: {operation}")),
    }
}

fn into_c_string(response: CoreResponse) -> *mut c_char {
    let json = serde_json::to_string(&response)
        .unwrap_or_else(|_| r#"{"ok":false,"error":"Could not encode core response."}"#.into());
    CString::new(json)
        .map(CString::into_raw)
        .unwrap_or_else(|_| ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn tether_core_request(request_json: *const c_char) -> *mut c_char {
    let response = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if request_json.is_null() {
            return CoreResponse::error("Request cannot be null.");
        }
        let request = match unsafe { CStr::from_ptr(request_json) }.to_str() {
            Ok(request) => request,
            Err(error) => return CoreResponse::error(format!("Request is not UTF-8: {error}")),
        };
        let request = match serde_json::from_str::<CoreRequest>(request) {
            Ok(request) => request,
            Err(error) => return CoreResponse::error(format!("Invalid request JSON: {error}")),
        };
        match dispatch_request(request) {
            Ok(data) => CoreResponse::success(data),
            Err(error) => CoreResponse::error(error),
        }
    }))
    .unwrap_or_else(|_| CoreResponse::error("The Tether core service failed unexpectedly."));
    into_c_string(response)
}

#[no_mangle]
pub extern "C" fn tether_audio_new(
    soundscape_index: u32,
    volume: f32,
    auto_play_on_focus: bool,
) -> *mut SoundscapePlayer {
    Box::into_raw(Box::new(SoundscapePlayer::new(
        SoundscapeType::from_index(soundscape_index),
        volume,
        auto_play_on_focus,
    )))
}

#[no_mangle]
pub unsafe extern "C" fn tether_audio_free(handle: *mut SoundscapePlayer) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

#[no_mangle]
pub unsafe extern "C" fn tether_audio_set_sound(
    handle: *mut SoundscapePlayer,
    soundscape_index: u32,
) -> bool {
    let Some(player) = (unsafe { handle.as_mut() }) else {
        return false;
    };
    player.set_sound(SoundscapeType::from_index(soundscape_index));
    true
}

#[no_mangle]
pub unsafe extern "C" fn tether_audio_set_volume(
    handle: *mut SoundscapePlayer,
    volume: f32,
) -> bool {
    let Some(player) = (unsafe { handle.as_mut() }) else {
        return false;
    };
    player.set_volume(volume);
    true
}

#[no_mangle]
pub unsafe extern "C" fn tether_audio_toggle(handle: *mut SoundscapePlayer) -> bool {
    let Some(player) = (unsafe { handle.as_mut() }) else {
        return false;
    };
    player.toggle_play();
    true
}

#[no_mangle]
pub unsafe extern "C" fn tether_audio_transition(
    handle: *mut SoundscapePlayer,
    old_state: u32,
    new_state: u32,
) -> bool {
    let Some(player) = (unsafe { handle.as_mut() }) else {
        return false;
    };
    let (Some(old_state), Some(new_state)) =
        (flow_state_from_code(old_state), flow_state_from_code(new_state))
    else {
        return false;
    };
    player.handle_flow_state_transition(old_state, new_state);
    true
}

#[no_mangle]
pub unsafe extern "C" fn tether_audio_chime(handle: *mut SoundscapePlayer, chime: u32) -> bool {
    let Some(player) = (unsafe { handle.as_mut() }) else {
        return false;
    };
    match chime {
        0 => player.play_chime(ChimeType::FocusStart),
        1 => player.play_chime(ChimeType::BreakStart),
        _ => return false,
    }
    true
}

fn flow_state_from_code(state: u32) -> Option<FlowState> {
    match state {
        0 => Some(FlowState::Idle),
        1 => Some(FlowState::MicroStart),
        2 => Some(FlowState::Focus),
        3 => Some(FlowState::Break),
        _ => None,
    }
}

#[no_mangle]
pub extern "C" fn tether_flow_new() -> *mut Flowmodoro {
    Box::into_raw(Box::new(Flowmodoro::new()))
}

#[no_mangle]
pub unsafe extern "C" fn tether_flow_free(handle: *mut Flowmodoro) {
    if !handle.is_null() {
        drop(unsafe { Box::from_raw(handle) });
    }
}

#[no_mangle]
pub unsafe extern "C" fn tether_flow_tick(handle: *mut Flowmodoro, elapsed_millis: u64) -> bool {
    let Some(flow) = (unsafe { handle.as_mut() }) else {
        return false;
    };
    flow.tick(Duration::from_millis(elapsed_millis));
    true
}

#[no_mangle]
pub unsafe extern "C" fn tether_flow_action(handle: *mut Flowmodoro, action: u32) -> bool {
    let Some(flow) = (unsafe { handle.as_mut() }) else {
        return false;
    };
    match action {
        0 => flow.start_focus(),
        1 => flow.start_micro_start(),
        2 => flow.toggle(),
        3 => flow.start_break(),
        4 => flow.resume_focus_from_break(),
        5 => flow.reset(),
        _ => return false,
    }
    true
}

#[no_mangle]
pub unsafe extern "C" fn tether_flow_set_ratios(
    handle: *mut Flowmodoro,
    focus_ratio: u32,
    break_ratio: u32,
) -> bool {
    let Some(flow) = (unsafe { handle.as_mut() }) else {
        return false;
    };
    flow.focus_ratio = focus_ratio.max(1);
    flow.break_ratio = break_ratio.max(1);
    true
}

#[no_mangle]
pub unsafe extern "C" fn tether_flow_snapshot(handle: *const Flowmodoro) -> *mut c_char {
    let Some(flow) = (unsafe { handle.as_ref() }) else {
        return ptr::null_mut();
    };
    let snapshot = FlowSnapshot::from(flow);
    let Ok(json) = serde_json::to_string(&snapshot) else {
        return ptr::null_mut();
    };
    CString::new(json)
        .map(CString::into_raw)
        .unwrap_or_else(|_| ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn tether_string_free(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn snapshot_uses_stable_json_fields_and_handles_invalid_actions() {
        let handle = Box::into_raw(Box::new(Flowmodoro::default()));
        unsafe {
            assert!(!tether_flow_action(handle, u32::MAX));
            assert!(tether_flow_set_ratios(handle, 0, 0));

            let snapshot = tether_flow_snapshot(handle);
            assert!(!snapshot.is_null());
            let json = CStr::from_ptr(snapshot).to_str().unwrap();
            let value: serde_json::Value = serde_json::from_str(json).unwrap();
            assert_eq!(value["state"], "idle");
            assert_eq!(value["time"], "00:00");
            assert_eq!(value["focus_ratio"], 1);
            assert_eq!(value["break_ratio"], 1);

            tether_string_free(snapshot);
            tether_flow_free(handle);
        }
    }

    #[test]
    fn service_api_returns_structured_errors_for_unknown_operations() {
        let request = CString::new(r#"{"operation":"missing.operation"}"#).unwrap();
        let response = unsafe { tether_core_request(request.as_ptr()) };
        assert!(!response.is_null());

        unsafe {
            let json = CStr::from_ptr(response).to_str().unwrap();
            let value: serde_json::Value = serde_json::from_str(json).unwrap();
            assert_eq!(value["ok"], false);
            assert_eq!(
                value["error"],
                "Unsupported core operation: missing.operation"
            );
            tether_string_free(response);
        }
    }
}
