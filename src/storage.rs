use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::types::{
    CampaignProgress, EmoteType, GameSettings, LocalUser, MatchRecord, ProfileCustomization,
    SkinType,
};

const SETTINGS_KEY: &str = "gm3d_rs_settings";
const PROFILE_KEY: &str = "gm3d_rs_profile";
const PROGRESS_KEY: &str = "gm3d_rs_progress";
const ACTIVE_SKIN_KEY: &str = "gm3d_rs_active_skin";
const ACTIVE_EMOTE_KEY: &str = "gm3d_rs_active_emote";
const USERS_KEY: &str = "gm3d_rs_users";
const ACTIVE_USER_KEY: &str = "gm3d_rs_active_user";
const MATCHES_KEY: &str = "gm3d_rs_matches";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub fn load_settings() -> GameSettings {
    load_json(SETTINGS_KEY).unwrap_or_default()
}

pub fn save_settings(settings: &GameSettings) {
    save_json(SETTINGS_KEY, settings);
}

pub fn load_profile() -> ProfileCustomization {
    load_json(PROFILE_KEY).unwrap_or_default()
}

pub fn save_profile(profile: &ProfileCustomization) {
    save_json(PROFILE_KEY, profile);
}

pub fn load_progress() -> CampaignProgress {
    load_json(PROGRESS_KEY).unwrap_or_default()
}

pub fn save_progress(progress: &CampaignProgress) {
    save_json(PROGRESS_KEY, progress);
}

pub fn load_active_skin() -> SkinType {
    load_json(ACTIVE_SKIN_KEY).unwrap_or(SkinType::Classic)
}

pub fn save_active_skin(skin: SkinType) {
    save_json(ACTIVE_SKIN_KEY, &skin);
}

pub fn load_active_emote() -> EmoteType {
    load_json(ACTIVE_EMOTE_KEY).unwrap_or(EmoteType::Bow)
}

pub fn save_active_emote(emote: EmoteType) {
    save_json(ACTIVE_EMOTE_KEY, &emote);
}

pub fn load_users() -> Vec<StoredUser> {
    load_json(USERS_KEY).unwrap_or_default()
}

pub fn save_users(users: &[StoredUser]) {
    save_json(USERS_KEY, users);
}

pub fn load_active_user() -> Option<LocalUser> {
    load_json(ACTIVE_USER_KEY)
}

pub fn save_active_user(user: Option<&LocalUser>) {
    if let Some(user) = user {
        save_json(ACTIVE_USER_KEY, user);
    } else {
        remove(ACTIVE_USER_KEY);
    }
}

pub fn load_matches(username: &str) -> Vec<MatchRecord> {
    let all: Vec<MatchRecord> = load_json(&matches_key(username)).unwrap_or_default();
    all
}

pub fn save_matches(username: &str, matches: &[MatchRecord]) {
    save_json(&matches_key(username), matches);
}

fn matches_key(username: &str) -> String {
    format!("{MATCHES_KEY}_{username}")
}

fn load_json<T: DeserializeOwned>(key: &str) -> Option<T> {
    let raw = storage()?.get_item(key).ok().flatten()?;
    serde_json::from_str(&raw).ok()
}

fn save_json<T: Serialize + ?Sized>(key: &str, value: &T) {
    if let (Some(storage), Ok(raw)) = (storage(), serde_json::to_string(value)) {
        let _ = storage.set_item(key, &raw);
    }
}

fn remove(key: &str) {
    if let Some(storage) = storage() {
        let _ = storage.remove_item(key);
    }
}

fn storage() -> Option<web_sys::Storage> {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()?.local_storage().ok().flatten()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}
