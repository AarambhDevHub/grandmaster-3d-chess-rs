use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerColor {
    White,
    Black,
}

impl PlayerColor {
    pub const fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::White => "White",
            Self::Black => "Black",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pawn => "P",
            Self::Knight => "N",
            Self::Bishop => "B",
            Self::Rook => "R",
            Self::Queen => "Q",
            Self::King => "K",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppScreen {
    Menu,
    Intro,
    Game,
    StoryMenu,
    Customization,
    PuzzleMenu,
    Auth,
    Profile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    PlayerVsPlayer,
    PlayerVsAi,
    Battle,
    Boss,
    Story,
    OnlineLocal,
    Puzzles,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub const fn depth(self) -> u8 {
        match self {
            Self::Easy => 1,
            Self::Medium => 2,
            Self::Hard => 3,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Medium => "Medium",
            Self::Hard => "Hard",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Classic,
    Volcano,
    Ice,
    Cyberpunk,
    Forest,
    Castle,
    Desert,
    Galaxy,
}

impl Theme {
    pub const ALL: [Self; 8] = [
        Self::Classic,
        Self::Volcano,
        Self::Ice,
        Self::Cyberpunk,
        Self::Forest,
        Self::Castle,
        Self::Desert,
        Self::Galaxy,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Volcano => "Volcano Arena",
            Self::Ice => "Ice Kingdom",
            Self::Cyberpunk => "Cyberpunk Neon",
            Self::Forest => "Magic Forest",
            Self::Castle => "Medieval Castle",
            Self::Desert => "Desert Ruins",
            Self::Galaxy => "Space Galaxy",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weather {
    Clear,
    Rain,
    Snow,
    Fog,
    Storm,
    Dust,
}

impl Weather {
    pub const ALL: [Self; 6] = [
        Self::Clear,
        Self::Rain,
        Self::Snow,
        Self::Fog,
        Self::Storm,
        Self::Dust,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Clear => "Clear",
            Self::Rain => "Rain",
            Self::Snow => "Snow",
            Self::Fog => "Fog",
            Self::Storm => "Storm",
            Self::Dust => "Dust",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeOfDay {
    Morning,
    Noon,
    Sunset,
    Night,
}

impl TimeOfDay {
    pub const ALL: [Self; 4] = [Self::Morning, Self::Noon, Self::Sunset, Self::Night];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Morning => "Morning",
            Self::Noon => "Noon",
            Self::Sunset => "Sunset",
            Self::Night => "Night",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardMutation {
    None,
    Hex,
    Giant,
    FogOfWar,
    Exploding,
    Portals,
}

impl BoardMutation {
    pub const ALL: [Self; 6] = [
        Self::None,
        Self::Hex,
        Self::Giant,
        Self::FogOfWar,
        Self::Exploding,
        Self::Portals,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Hex => "Hex",
            Self::Giant => "Giant",
            Self::FogOfWar => "FogOfWar",
            Self::Exploding => "Exploding",
            Self::Portals => "Portals",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphicsQuality {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraMode {
    Orbit,
    Top,
    Cinematic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CinematicMode {
    None,
    KillCam,
    GiantKingEnding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkinType {
    Classic,
    Marble,
    Gold,
    Crystal,
    Bronze,
    Shadow,
    Neon,
    Wood,
}

impl SkinType {
    pub const ALL: [Self; 8] = [
        Self::Classic,
        Self::Marble,
        Self::Gold,
        Self::Crystal,
        Self::Bronze,
        Self::Shadow,
        Self::Neon,
        Self::Wood,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Marble => "Marble",
            Self::Gold => "Gold",
            Self::Crystal => "Crystal",
            Self::Bronze => "Bronze",
            Self::Shadow => "Shadow",
            Self::Neon => "Neon",
            Self::Wood => "Wood",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmoteType {
    None,
    Bow,
    Salute,
    Spin,
    Power,
}

impl EmoteType {
    pub const ALL: [Self; 5] = [Self::None, Self::Bow, Self::Salute, Self::Spin, Self::Power];

    pub const fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Bow => "Bow",
            Self::Salute => "Salute",
            Self::Spin => "Spin",
            Self::Power => "Power",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSettings {
    pub show_animations: bool,
    pub show_vfx: bool,
    pub cinematic_camera: bool,
    pub enable_cutscenes: bool,
    pub graphics_quality: GraphicsQuality,
    pub theme: Theme,
    pub enable_coach: bool,
    pub enable_sounds: bool,
    pub enable_voice_overs: bool,
    pub enable_story_vo: bool,
    pub enable_emotes: bool,
    pub performance_mode: bool,
    pub weather: Weather,
    pub time_of_day: TimeOfDay,
    pub enable_day_night_cycle: bool,
    pub board_mutation: BoardMutation,
    pub enable_physics_death: bool,
    pub enable_magic_skills: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            show_animations: true,
            show_vfx: true,
            cinematic_camera: true,
            enable_cutscenes: true,
            graphics_quality: GraphicsQuality::Medium,
            theme: Theme::Classic,
            enable_coach: false,
            enable_sounds: true,
            enable_voice_overs: true,
            enable_story_vo: true,
            enable_emotes: true,
            performance_mode: false,
            weather: Weather::Clear,
            time_of_day: TimeOfDay::Noon,
            enable_day_night_cycle: false,
            board_mutation: BoardMutation::None,
            enable_physics_death: false,
            enable_magic_skills: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileCustomization {
    pub icon: String,
    pub banner_color: String,
    pub emblem: String,
    pub crown_style: String,
}

impl Default for ProfileCustomization {
    fn default() -> Self {
        Self {
            icon: "👑".to_string(),
            banner_color: "#4f46e5".to_string(),
            emblem: "Lion".to_string(),
            crown_style: "Gold".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignProgress {
    pub current_chapter: u32,
    pub unlocked_skins: Vec<SkinType>,
    pub unlocked_emotes: Vec<EmoteType>,
    pub completed_chapters: Vec<u32>,
}

impl Default for CampaignProgress {
    fn default() -> Self {
        Self {
            current_chapter: 1,
            unlocked_skins: SkinType::ALL.to_vec(),
            unlocked_emotes: vec![EmoteType::Bow],
            completed_chapters: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StoryChapter {
    pub id: u32,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub description: &'static str,
    pub theme: Theme,
    pub difficulty: Difficulty,
    pub enemy_name: &'static str,
    pub intro_text: &'static str,
    pub reward_skin: Option<SkinType>,
}

#[derive(Clone, Copy, Debug)]
pub struct Puzzle {
    pub id: &'static str,
    pub fen: &'static str,
    pub moves: &'static [&'static str],
    pub rating: u16,
    pub description: &'static str,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchRecord {
    pub id: u64,
    pub opponent: String,
    pub result: String,
    pub moves: String,
    pub date: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserStats {
    pub username: String,
    pub elo: i32,
    pub total_matches: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalUser {
    pub username: String,
    pub email: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BossState {
    pub is_active: bool,
    pub boss_hp: i32,
    pub max_hp: i32,
    pub player_hp: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveData {
    pub from: String,
    pub to: String,
    pub promotion: Option<PieceKind>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LastAction {
    pub action_type: String,
    pub from: String,
    pub to: String,
    pub piece: PieceKind,
    pub captured_piece: Option<PieceKind>,
    pub captured_square: Option<String>,
    pub is_check: bool,
    pub is_checkmate: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChessPieceView {
    pub kind: PieceKind,
    pub color: PlayerColor,
    pub square: String,
}
