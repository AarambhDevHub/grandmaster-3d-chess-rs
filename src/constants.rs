use crate::types::{Difficulty, EmoteType, Puzzle, SkinType, StoryChapter, Theme, Weather};

pub const BOARD_SIZE: usize = 8;
pub const TILE_SIZE: f32 = 1.5;
pub const BOARD_OFFSET: f32 = (BOARD_SIZE as f32 * TILE_SIZE) / 2.0 - TILE_SIZE / 2.0;

pub const PROFILE_ICONS: [&str; 8] = ["👑", "⚔️", "🛡️", "🐉", "🔮", "♘", "🏰", "💀"];
pub const BANNER_COLORS: [&str; 6] = [
    "#4f46e5", "#dc2626", "#16a34a", "#d97706", "#9333ea", "#2563eb",
];

#[derive(Clone, Copy)]
pub struct ThemeConfig {
    pub light: u32,
    pub dark: u32,
    pub bg: u32,
    pub accent: u32,
    pub ambient: f32,
}

pub fn theme_config(theme: Theme) -> ThemeConfig {
    match theme {
        Theme::Classic => ThemeConfig {
            light: 0x94a3b8,
            dark: 0x475569,
            bg: 0x0f172a,
            accent: 0x22c55e,
            ambient: 0.5,
        },
        Theme::Volcano => ThemeConfig {
            light: 0x7f1d1d,
            dark: 0x450a0a,
            bg: 0x2a0a0a,
            accent: 0xef4444,
            ambient: 0.2,
        },
        Theme::Ice => ThemeConfig {
            light: 0xe0f2fe,
            dark: 0x0ea5e9,
            bg: 0x082f49,
            accent: 0x38bdf8,
            ambient: 0.8,
        },
        Theme::Cyberpunk => ThemeConfig {
            light: 0x18181b,
            dark: 0x09090b,
            bg: 0x000000,
            accent: 0xd946ef,
            ambient: 0.1,
        },
        Theme::Forest => ThemeConfig {
            light: 0x86efac,
            dark: 0x14532d,
            bg: 0x052e16,
            accent: 0x4ade80,
            ambient: 0.4,
        },
        Theme::Castle => ThemeConfig {
            light: 0xd4d4d8,
            dark: 0x52525b,
            bg: 0x27272a,
            accent: 0xf59e0b,
            ambient: 0.3,
        },
        Theme::Desert => ThemeConfig {
            light: 0xfde68a,
            dark: 0xd97706,
            bg: 0x78350f,
            accent: 0xfbbf24,
            ambient: 0.6,
        },
        Theme::Galaxy => ThemeConfig {
            light: 0xc084fc,
            dark: 0x3b0764,
            bg: 0x000000,
            accent: 0x818cf8,
            ambient: 0.15,
        },
    }
}

#[derive(Clone, Copy)]
pub struct WeatherConfig {
    pub particle_count: usize,
    pub color: u32,
    pub speed: f32,
}

pub fn weather_config(weather: Weather) -> WeatherConfig {
    match weather {
        Weather::Clear => WeatherConfig {
            particle_count: 0,
            color: 0xffffff,
            speed: 0.0,
        },
        Weather::Rain => WeatherConfig {
            particle_count: 180,
            color: 0xa5f3fc,
            speed: 1.5,
        },
        Weather::Snow => WeatherConfig {
            particle_count: 140,
            color: 0xffffff,
            speed: 0.25,
        },
        Weather::Fog => WeatherConfig {
            particle_count: 80,
            color: 0xcbd5e1,
            speed: 0.05,
        },
        Weather::Storm => WeatherConfig {
            particle_count: 260,
            color: 0x67e8f9,
            speed: 2.0,
        },
        Weather::Dust => WeatherConfig {
            particle_count: 110,
            color: 0xfdba74,
            speed: 0.4,
        },
    }
}

#[derive(Clone, Copy)]
pub struct SkinConfig {
    pub white: u32,
    pub black: u32,
    pub emissive_white: u32,
    pub emissive_black: u32,
    pub roughness: f32,
    pub metalness: f32,
}

pub fn skin_config(skin: SkinType) -> SkinConfig {
    match skin {
        SkinType::Classic => SkinConfig {
            white: 0xe2e8f0,
            black: 0x1e293b,
            emissive_white: 0x111111,
            emissive_black: 0x000000,
            roughness: 0.5,
            metalness: 0.1,
        },
        SkinType::Marble => SkinConfig {
            white: 0xf8fafc,
            black: 0x334155,
            emissive_white: 0xffffff,
            emissive_black: 0x0f172a,
            roughness: 0.1,
            metalness: 0.0,
        },
        SkinType::Gold => SkinConfig {
            white: 0xfcd34d,
            black: 0x1e1b4b,
            emissive_white: 0xb45309,
            emissive_black: 0x000000,
            roughness: 0.2,
            metalness: 1.0,
        },
        SkinType::Crystal => SkinConfig {
            white: 0xe0f2fe,
            black: 0x475569,
            emissive_white: 0xbae6fd,
            emissive_black: 0x0f172a,
            roughness: 0.0,
            metalness: 0.1,
        },
        SkinType::Bronze => SkinConfig {
            white: 0xd97706,
            black: 0x78350f,
            emissive_white: 0x92400e,
            emissive_black: 0x451a03,
            roughness: 0.6,
            metalness: 0.8,
        },
        SkinType::Shadow => SkinConfig {
            white: 0x94a3b8,
            black: 0x020617,
            emissive_white: 0x475569,
            emissive_black: 0x000000,
            roughness: 0.9,
            metalness: 0.2,
        },
        SkinType::Neon => SkinConfig {
            white: 0x22d3ee,
            black: 0xf472b6,
            emissive_white: 0x0891b2,
            emissive_black: 0xbe185d,
            roughness: 0.2,
            metalness: 0.5,
        },
        SkinType::Wood => SkinConfig {
            white: 0xfde68a,
            black: 0x92400e,
            emissive_white: 0xd97706,
            emissive_black: 0x78350f,
            roughness: 0.8,
            metalness: 0.0,
        },
    }
}

pub const STORY_CHAPTERS: [StoryChapter; 5] = [
    StoryChapter {
        id: 1,
        title: "Rise of the Marble Kingdom",
        subtitle: "The Beginning",
        description: "Begin your journey as a young tactician in the ancient halls of the Marble King.",
        theme: Theme::Classic,
        difficulty: Difficulty::Easy,
        enemy_name: "Marble King",
        intro_text: "Long ago, strategy was the language of kings. Prove your worth in the Marble Halls.",
        reward_skin: Some(SkinType::Marble),
    },
    StoryChapter {
        id: 2,
        title: "Shadow of the Obsidian Empire",
        subtitle: "Into Darkness",
        description: "The Obsidian Empire has risen from the depths. Their pieces are forged in darkness.",
        theme: Theme::Volcano,
        difficulty: Difficulty::Medium,
        enemy_name: "Obsidian Warlord",
        intro_text: "Darkness gathers. The Obsidian army knows no mercy.",
        reward_skin: Some(SkinType::Shadow),
    },
    StoryChapter {
        id: 3,
        title: "The Crystal Rebellion",
        subtitle: "Shattered Peace",
        description: "In the frozen north, the Crystal Rebellion challenges the throne.",
        theme: Theme::Ice,
        difficulty: Difficulty::Medium,
        enemy_name: "Crystal Queen",
        intro_text: "Cold as ice, sharp as glass. Do not let your resolve shatter.",
        reward_skin: Some(SkinType::Crystal),
    },
    StoryChapter {
        id: 4,
        title: "Bronze Age Warfront",
        subtitle: "Fires of War",
        description: "The desert winds carry the sound of clashing bronze. A general awaits.",
        theme: Theme::Desert,
        difficulty: Difficulty::Hard,
        enemy_name: "Bronze General",
        intro_text: "War is heat. War is metal. Survive the desert's wrath.",
        reward_skin: Some(SkinType::Bronze),
    },
    StoryChapter {
        id: 5,
        title: "The Final Neon Dawn",
        subtitle: "Future's End",
        description: "Travel to the edge of time. Defeat the Eternal Neon King to claim the crown of history.",
        theme: Theme::Cyberpunk,
        difficulty: Difficulty::Hard,
        enemy_name: "Eternal Neon King",
        intro_text: "The future is written in light. This is your final test.",
        reward_skin: Some(SkinType::Neon),
    },
];

pub const SAMPLE_PUZZLES: [Puzzle; 5] = [
    Puzzle {
        id: "1",
        fen: "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4",
        moves: &["h5f7"],
        rating: 800,
        description: "Scholar's Mate (Mate in 1)",
    },
    Puzzle {
        id: "2",
        fen: "rnbqkbnr/ppppp2p/5p2/6p1/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3",
        moves: &["d1h5"],
        rating: 900,
        description: "Fool's Mate Pattern (Mate in 1)",
    },
    Puzzle {
        id: "3",
        fen: "3r4/pr3pkp/1q2p1p1/8/B7/1P2P3/P4PPP/2R2RK1 w - - 0 1",
        moves: &["c1c6", "b6a5", "c6a6"],
        rating: 1200,
        description: "Skewer Tactic",
    },
    Puzzle {
        id: "4",
        fen: "r1b1k2r/ppppqppp/2n2n2/4p3/1bB1P3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 4 5",
        moves: &["c3d5", "f6d5", "e4d5"],
        rating: 1100,
        description: "Center Control",
    },
    Puzzle {
        id: "5",
        fen: "r1bqk2r/pppp1ppp/2n2n2/4p3/1bB1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
        moves: &["e1g1", "f6e4", "d2d4"],
        rating: 1300,
        description: "Italian Game Gambit",
    },
];

pub const ALL_EMOTES: [EmoteType; 5] = EmoteType::ALL;
