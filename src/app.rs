use leptos::prelude::*;

use crate::audio::AudioSystem;
use crate::constants::STORY_CHAPTERS;
use crate::game::GameController;
use crate::storage::{
    StoredUser, load_active_emote, load_active_skin, load_active_user, load_matches, load_profile,
    load_progress, load_settings, load_users, save_active_emote, save_active_skin,
    save_active_user, save_matches, save_profile, save_progress, save_settings, save_users,
};
use crate::types::{
    AppScreen, CampaignProgress, Difficulty, EmoteType, GameMode, GameSettings, LocalUser,
    MatchRecord, PlayerColor, ProfileCustomization, SkinType, UserStats,
};

#[derive(Clone)]
pub struct AppContext {
    pub state: RwSignal<AppModel>,
    pub audio: AudioSystem,
}

#[derive(Clone, Debug)]
pub struct AppModel {
    pub screen: AppScreen,
    pub player_name: String,
    pub profile: ProfileCustomization,
    pub active_skin: SkinType,
    pub active_emote: EmoteType,
    pub campaign_progress: CampaignProgress,
    pub settings: GameSettings,
    pub show_settings: bool,
    pub user: Option<LocalUser>,
    pub matches: Vec<MatchRecord>,
    pub game: GameController,
    pub auth_error: Option<String>,
    pub renderer_error: Option<String>,
    recorded_fen_count: usize,
}

impl AppModel {
    pub fn load() -> Self {
        let user = load_active_user();
        let matches = user
            .as_ref()
            .map(|user| load_matches(&user.username))
            .unwrap_or_default();
        Self {
            screen: AppScreen::Menu,
            player_name: user
                .as_ref()
                .map(|user| user.username.clone())
                .unwrap_or_else(|| "Player".to_string()),
            profile: load_profile(),
            active_skin: load_active_skin(),
            active_emote: load_active_emote(),
            campaign_progress: load_progress(),
            settings: load_settings(),
            show_settings: false,
            user,
            matches,
            game: GameController::default(),
            auth_error: None,
            renderer_error: None,
            recorded_fen_count: 0,
        }
    }

    pub fn stats(&self) -> UserStats {
        let username = self
            .user
            .as_ref()
            .map(|user| user.username.clone())
            .unwrap_or_else(|| self.player_name.clone());
        let wins = self.matches.iter().filter(|m| m.result == "win").count() as u32;
        let losses = self.matches.iter().filter(|m| m.result == "loss").count() as u32;
        let draws = self.matches.iter().filter(|m| m.result == "draw").count() as u32;
        let total = self.matches.len() as u32;
        UserStats {
            username,
            elo: 1200 + wins as i32 * 18 - losses as i32 * 14,
            total_matches: total,
            wins,
            losses,
            draws,
        }
    }

    pub fn set_screen(&mut self, screen: AppScreen) {
        self.screen = screen;
    }

    pub fn start_game(&mut self, mode: GameMode, difficulty: Difficulty) {
        self.game.init(mode, difficulty, &self.player_name);
        self.recorded_fen_count = 0;
        self.screen = AppScreen::Game;
    }

    pub fn start_story(&mut self, chapter_id: u32) {
        if let Some(chapter) = STORY_CHAPTERS
            .iter()
            .find(|chapter| chapter.id == chapter_id)
        {
            self.settings.theme = chapter.theme;
            save_settings(&self.settings);
        }
        self.game.start_story_chapter(chapter_id, &self.player_name);
        self.recorded_fen_count = 0;
        self.screen = AppScreen::Game;
    }

    pub fn update_settings(&mut self, update: impl FnOnce(&mut GameSettings)) {
        update(&mut self.settings);
        save_settings(&self.settings);
    }

    pub fn set_profile(&mut self, update: impl FnOnce(&mut ProfileCustomization)) {
        update(&mut self.profile);
        save_profile(&self.profile);
    }

    pub fn set_skin(&mut self, skin: SkinType) {
        self.active_skin = skin;
        save_active_skin(skin);
    }

    pub fn set_emote(&mut self, emote: EmoteType) {
        self.active_emote = emote;
        save_active_emote(emote);
    }

    pub fn login(&mut self, username: &str, password: &str) -> bool {
        let users = load_users();
        let found = users
            .into_iter()
            .find(|user| user.username == username && user.password == password);
        if let Some(user) = found {
            let local = LocalUser {
                username: user.username,
                email: user.email,
            };
            self.player_name = local.username.clone();
            self.matches = load_matches(&local.username);
            self.user = Some(local.clone());
            self.auth_error = None;
            save_active_user(Some(&local));
            self.screen = AppScreen::Menu;
            true
        } else {
            self.auth_error = Some("Invalid local username or password.".to_string());
            false
        }
    }

    pub fn register(&mut self, username: &str, email: &str, password: &str) -> bool {
        let mut users = load_users();
        if users.iter().any(|user| user.username == username) {
            self.auth_error = Some("That local username already exists.".to_string());
            return false;
        }
        users.push(StoredUser {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        });
        save_users(&users);
        self.login(username, password)
    }

    pub fn after_move(&mut self) {
        if self.game.is_game_over {
            self.record_match_once();
            self.unlock_story_if_needed();
        }
    }

    fn record_match_once(&mut self) {
        if self.game.history_fen.len() == self.recorded_fen_count
            || self.game.history_uci.is_empty()
        {
            return;
        }
        self.recorded_fen_count = self.game.history_fen.len();
        let Some(user) = &self.user else {
            return;
        };
        let result = match self.game.winner {
            Some(winner) if winner == self.game.player_color => "win",
            Some(_) => "loss",
            None => "draw",
        }
        .to_string();
        let opponent = match self.game.game_mode {
            GameMode::PlayerVsAi => "Rust Bot",
            GameMode::Story => "Story Opponent",
            GameMode::Boss => "Boss Rush",
            GameMode::OnlineLocal => "Local Room",
            GameMode::Puzzles => "Puzzle",
            _ => "Local Player",
        }
        .to_string();
        let record = MatchRecord {
            id: current_id(),
            opponent,
            result,
            moves: self.game.history_uci.join(" "),
            date: current_date_label(),
        };
        self.matches.insert(0, record);
        save_matches(&user.username, &self.matches);
    }

    fn unlock_story_if_needed(&mut self) {
        if self.game.game_mode != GameMode::Story || self.game.winner != Some(PlayerColor::White) {
            return;
        }
        let chapter_id = self.campaign_progress.current_chapter;
        if self
            .campaign_progress
            .completed_chapters
            .contains(&chapter_id)
        {
            return;
        }
        self.campaign_progress.completed_chapters.push(chapter_id);
        self.campaign_progress.current_chapter += 1;
        if let Some(chapter) = STORY_CHAPTERS
            .iter()
            .find(|chapter| chapter.id == chapter_id)
        {
            if let Some(skin) = chapter.reward_skin {
                if !self.campaign_progress.unlocked_skins.contains(&skin) {
                    self.campaign_progress.unlocked_skins.push(skin);
                }
            }
        }
        save_progress(&self.campaign_progress);
    }
}

fn current_id() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        1
    }
}

fn current_date_label() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::new_0().to_iso_string().into()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "local".to_string()
    }
}
