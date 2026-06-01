use leptos::prelude::*;

use crate::app::AppContext;
use crate::constants::{
    ALL_EMOTES, BANNER_COLORS, PROFILE_ICONS, SAMPLE_PUZZLES, STORY_CHAPTERS, theme_config,
};
use crate::types::{
    AppScreen, CameraMode, Difficulty, GameMode, GraphicsQuality, PieceKind, SkinType, Theme,
    TimeOfDay, Weather,
};

#[component]
pub fn Interface() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <div class="ui-layer">
            {move || {
                let model = ctx.state.get();
                match model.screen {
                    AppScreen::Menu => view! { <MainMenu /> }.into_any(),
                    AppScreen::Intro => view! { <IntroScreen /> }.into_any(),
                    AppScreen::Game => view! {
                        <Hud />
                        <PromotionPopup />
                        <VictoryScreen />
                    }.into_any(),
                    AppScreen::StoryMenu => view! { <StoryMenu /> }.into_any(),
                    AppScreen::Customization => view! { <CustomizationMenu /> }.into_any(),
                    AppScreen::PuzzleMenu => view! { <PuzzleMenu /> }.into_any(),
                    AppScreen::Auth => view! { <AuthMenu /> }.into_any(),
                    AppScreen::Profile => view! { <ProfilePage /> }.into_any(),
                }
            }}
            {move || if ctx.state.get().show_settings {
                view! { <SettingsModal /> }.into_any()
            } else {
                view! {}.into_any()
            }}
            {move || ctx.state.get().renderer_error.map(|error| view! {
                <div class="error-banner" style="position:absolute;left:16px;bottom:16px;z-index:80;max-width:460px;">
                    {format!("Renderer error: {error}")}
                </div>
            })}
        </div>
    }
}

#[component]
fn IntroScreen() -> impl IntoView {
    view! {
        <div class="screen">
            <div class="title-stack">
                <div class="crown-mark">"K"</div>
                <h1 class="hero-title">"ROYAL "<span>"3D"</span>" CHESS"</h1>
                <p class="hero-subtitle">"Cinematic board deployment"</p>
            </div>
        </div>
    }
}

#[component]
fn MainMenu() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let (view_mode, set_view_mode) = signal("main".to_string());
    let (temp_name, set_temp_name) = signal(ctx.state.get_untracked().player_name);
    let (difficulty, set_difficulty) = signal(Difficulty::Medium);

    view! {
        {move || match view_mode.get().as_str() {
            "setup" => view! {
                <div class="modal-backdrop">
                    <div class="panel setup-panel">
                        <div class="panel-accent"></div>
                        <div class="panel-header center-row">
                            <IconCrown />
                            <h2>"Identify Yourself"</h2>
                        </div>
                        <div class="panel-body stack">
                            <p class="muted-center">"Choose your name and enter the royal board."</p>
                            <input
                                class="input"
                                maxlength="12"
                                prop:value=move || temp_name.get()
                                on:input=move |ev| set_temp_name.set(event_target_value(&ev))
                                placeholder="Enter name"
                            />
                            <div class="button-row center-row">
                                <button class=move || difficulty_class(difficulty.get(), Difficulty::Easy) on:click=move |_| set_difficulty.set(Difficulty::Easy)>{Difficulty::Easy.label()}</button>
                                <button class=move || difficulty_class(difficulty.get(), Difficulty::Medium) on:click=move |_| set_difficulty.set(Difficulty::Medium)>{Difficulty::Medium.label()}</button>
                                <button class=move || difficulty_class(difficulty.get(), Difficulty::Hard) on:click=move |_| set_difficulty.set(Difficulty::Hard)>{Difficulty::Hard.label()}</button>
                            </div>
                            <button class="btn btn-emerald mode-button" on:click=move |_| {
                                ctx.state.update(|model| {
                                    model.player_name = name_or_player(&temp_name.get());
                                    model.start_game(GameMode::PlayerVsAi, difficulty.get());
                                });
                            }><IconMonitor />"VS AI"</button>
                            <button class="btn mode-button" on:click=move |_| {
                                ctx.state.update(|model| {
                                    model.player_name = name_or_player(&temp_name.get());
                                    model.start_game(GameMode::PlayerVsPlayer, Difficulty::Medium);
                                });
                            }><IconUsers />"Player vs Player"</button>
                            <button class="btn btn-ghost" on:click=move |_| set_view_mode.set("main".to_string())>"Back"</button>
                        </div>
                    </div>
                </div>
            }.into_any(),
            _ => view! {
                <div class="screen">
                    <div class="title-stack">
                        <div class="crown-mark"><IconCrown /></div>
                        <h1 class="hero-title">"ROYAL "<span>"3D"</span>" CHESS"</h1>
                        <p class="hero-subtitle">"The Ultimate Strategy Experience"</p>
                        <div class="menu-actions">
                            <button class="btn btn-primary play-button" on:click=move |_| set_view_mode.set("setup".to_string())>
                                "PLAY"
                                <IconArrowRight />
                            </button>
                            <div class="button-row center-row">
                                <button class="btn pill-nav customize-nav" on:click=move |_| ctx.state.update(|m| m.set_screen(AppScreen::Customization))>
                                    <IconPalette />"Customize"
                                </button>
                            </div>
                            <button class="btn btn-ghost settings-link" on:click=move |_| ctx.state.update(|m| m.show_settings = true)>
                                <IconSettings />"Settings"
                            </button>
                        </div>
                    </div>
                    <div class="footer-version">
                        <span>"VERSION 1.0"</span>
                        <span>"- "</span>
                        <a href="https://leptos.dev/" target="_blank" rel="noopener noreferrer">"LEPTOS"</a>
                        <span>"- "</span>
                        <a href="https://github.com/AarambhDevHub/scenix" target="_blank" rel="noopener noreferrer">"SCENIX"</a>
                        <span>"- "</span>
                        <a href="https://github.com/AarambhDevHub/animato" target="_blank" rel="noopener noreferrer">"ANIMATO"</a>
                    </div>
                </div>
            }.into_any(),
        }}
    }
}

#[component]
fn IconCrown() -> impl IntoView {
    view! {
        <svg class="ui-icon crown-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M3.5 18.2h17M5 15.2 3.4 6.4l5.2 4.1L12 3.8l3.4 6.7 5.2-4.1-1.6 8.8H5Z" />
            <path d="M5.8 21h12.4" />
        </svg>
    }
}

#[component]
fn IconArrowRight() -> impl IntoView {
    view! {
        <svg class="ui-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M5 12h13" />
            <path d="m13 6 6 6-6 6" />
        </svg>
    }
}

#[component]
fn IconPalette() -> impl IntoView {
    view! {
        <svg class="ui-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M12 22a10 10 0 1 1 10-10c0 2.2-1.5 3-3.1 3h-1.7a1.7 1.7 0 0 0-1.7 1.7c0 .5.2.9.5 1.3.4.4.5.8.5 1.3 0 1.8-2.1 2.7-4.5 2.7Z" />
            <path d="M7.5 10.4h.1M10 7.2h.1M14.1 7.2h.1M16.5 10.4h.1" />
        </svg>
    }
}

#[component]
fn IconSettings() -> impl IntoView {
    view! {
        <svg class="ui-icon settings-gear" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M9.7 3.4 10.4 2h3.2l.7 1.4c.6.2 1.1.4 1.7.7l1.5-.5 2.2 2.2-.5 1.5c.3.5.5 1.1.7 1.7l1.4.7v3.2l-1.4.7a7.5 7.5 0 0 1-.7 1.7l.5 1.5-2.2 2.2-1.5-.5c-.5.3-1.1.5-1.7.7l-.7 1.4h-3.2l-.7-1.4a7.5 7.5 0 0 1-1.7-.7l-1.5.5-2.2-2.2.5-1.5a7.5 7.5 0 0 1-.7-1.7L2.7 13V9.8l1.4-.7c.2-.6.4-1.2.7-1.7l-.5-1.5 2.2-2.2 1.5.5c.5-.4 1.1-.6 1.7-.8Z" />
            <circle cx="12" cy="11.4" r="3" />
        </svg>
    }
}

#[component]
fn IconMonitor() -> impl IntoView {
    view! {
        <svg class="ui-icon" viewBox="0 0 24 24" aria-hidden="true">
            <rect x="3" y="4" width="18" height="12" rx="2" />
            <path d="M8 21h8M12 16v5" />
        </svg>
    }
}

#[component]
fn IconUsers() -> impl IntoView {
    view! {
        <svg class="ui-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M16 21v-2a4 4 0 0 0-4-4H7a4 4 0 0 0-4 4v2" />
            <circle cx="9.5" cy="7" r="4" />
            <path d="M22 21v-2a4 4 0 0 0-3-3.9M16 3.2a4 4 0 0 1 0 7.6" />
        </svg>
    }
}

#[component]
fn Hud() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <div class="hud">
            <div class="hud-top">
                <button class="btn icon-btn" title="Exit" on:click=move |_| ctx.state.update(|m| m.set_screen(AppScreen::Menu))>"<"</button>
                <div>
                    <div class="status-pill">
                        <div class="status-title">
                            {move || {
                                let model = ctx.state.get();
                                if model.game.is_replay_mode {
                                    "REPLAY MODE".to_string()
                                } else if model.game.boss_state.is_active {
                                    "BOSS FIGHT".to_string()
                                } else if model.game.game_mode == GameMode::Puzzles {
                                    "PUZZLE MODE".to_string()
                                } else {
                                    format!("{}'s Turn", model.game.turn.label())
                                }
                            }}
                            {move || if ctx.state.get().game.is_check { " - CHECK".to_string() } else { String::new() }}
                        </div>
                        <div class="status-sub">{move || ctx.state.get().game.commentary}</div>
                    </div>
                    {move || {
                        let boss = ctx.state.get().game.boss_state;
                        if boss.is_active {
                            let pct = if boss.max_hp > 0 { (boss.boss_hp as f32 / boss.max_hp as f32 * 100.0).max(0.0) } else { 0.0 };
                            view! {
                                <div class="boss-bar">
                                    <div class="boss-fill" style=format!("width:{pct:.1}%")></div>
                                    <span>{format!("BOSS HP: {} / {}", boss.boss_hp, boss.max_hp)}</span>
                                </div>
                            }.into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}
                </div>
                <div class="row">
                    <button class="btn icon-btn" title="Orbit" on:click=move |_| ctx.state.update(|m| m.game.camera_mode = CameraMode::Orbit)>"O"</button>
                    <button class="btn icon-btn" title="Top" on:click=move |_| ctx.state.update(|m| m.game.camera_mode = CameraMode::Top)>"T"</button>
                    <button class="btn icon-btn" title="Settings" on:click=move |_| ctx.state.update(|m| m.show_settings = true)><IconSettings /></button>
                    {move || if ctx.state.get().game.game_mode == GameMode::Puzzles && !ctx.state.get().game.is_puzzle_solved {
                        view! { <button class="btn icon-btn btn-indigo" title="Hint" on:click=move |_| ctx.state.update(|m| m.game.show_puzzle_hint())>"?"</button> }.into_any()
                    } else { view! {}.into_any() }}
                </div>
            </div>

            <div class="captured">
                <div class="captured-row">{move || format!("White captures: {}", piece_list(&ctx.state.get().game.captured.white))}</div>
                <div class="captured-row">{move || format!("Black captures: {}", piece_list(&ctx.state.get().game.captured.black))}</div>
            </div>

            <div class="commentary">
                <small>"Live Commentary"</small>
                {move || ctx.state.get().game.commentary}
            </div>

            <div class="hud-bottom">
                <div class="row">
                    {move || if !ctx.state.get().game.is_replay_mode {
                        view! {
                            <button class="btn icon-btn" title="Undo" on:click=move |_| ctx.state.update(|m| m.game.undo())>"U"</button>
                            <button class="btn icon-btn" title="Reset" on:click=move |_| {
                                let (mode, difficulty) = {
                                    let model = ctx.state.get_untracked();
                                    (model.game.game_mode, model.game.difficulty)
                                };
                                ctx.state.update(|m| m.start_game(mode, difficulty));
                            }>"R"</button>
                            {move || if ctx.state.get().game.is_game_over {
                                view! { <button class="btn icon-btn btn-indigo" title="Replay" on:click=move |_| ctx.state.update(|m| m.game.start_replay())>"P"</button> }.into_any()
                            } else { view! {}.into_any() }}
                        }.into_any()
                    } else {
                        view! {
                            <button class="btn icon-btn" on:click=move |_| ctx.state.update(|m| m.game.prev_replay_move())>"<"</button>
                            <button class="btn icon-btn" on:click=move |_| ctx.state.update(|m| m.game.next_replay_move())>">"</button>
                            <button class="btn btn-red" on:click=move |_| ctx.state.update(|m| m.game.stop_replay())>"Exit Replay"</button>
                        }.into_any()
                    }}
                </div>
                <MoveHistory />
            </div>
        </div>
    }
}

#[component]
fn MoveHistory() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <div class="history">
            <div class="history-title">"Move History"</div>
            <div class="history-body">
                {move || {
                    let moves = ctx.state.get().game.history;
                    moves.chunks(2)
                        .enumerate()
                        .map(|(index, pair)| {
                            let white = pair.first().cloned().unwrap_or_default();
                            let black = pair.get(1).cloned().unwrap_or_default();
                            view! {
                                <div class="move-line">
                                    <span>{format!("{}.", index + 1)}</span>
                                    <span>{white}</span>
                                    <span>{black}</span>
                                </div>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }
}

#[component]
fn PromotionPopup() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        {move || if ctx.state.get().game.promotion_pending.is_some() {
            view! {
                <div class="modal-backdrop">
                    <div class="panel" style="width:min(94vw,420px);">
                        <div class="panel-header center-row"><h2>"Promote Pawn"</h2></div>
                        <div class="panel-body button-row center-row">
                            <PromotionButton kind=PieceKind::Queen label="Queen" />
                            <PromotionButton kind=PieceKind::Rook label="Rook" />
                            <PromotionButton kind=PieceKind::Bishop label="Bishop" />
                            <PromotionButton kind=PieceKind::Knight label="Knight" />
                        </div>
                    </div>
                </div>
            }.into_any()
        } else {
            view! {}.into_any()
        }}
    }
}

#[component]
fn PromotionButton(kind: PieceKind, label: &'static str) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <button class="btn" on:click=move |_| {
            let mut moved = false;
            ctx.state.update(|model| {
                moved = model.game.set_promotion(kind);
                if moved {
                    model.after_move();
                }
            });
            if moved {
                ctx.audio.play_move(ctx.state.get_untracked().settings.enable_sounds);
            }
        }>{label}</button>
    }
}

#[component]
fn VictoryScreen() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        {move || {
            let model = ctx.state.get();
            if !model.game.is_game_over {
                return view! {}.into_any();
            }
            let message = model.game.winner
                .map(|winner| format!("{} Wins!", winner.label()))
                .unwrap_or_else(|| "Draw!".to_string());
            view! {
                <div class="modal-backdrop">
                    <div class="panel" style="width:min(94vw,520px);text-align:center;">
                        <div class="panel-body stack">
                            <h1 style="font-family:Georgia,serif;font-size:48px;color:#fbbf24;margin:0;">{message}</h1>
                            <p style="color:#94a3b8;margin:0 0 12px;">{if model.game.winner.is_some() { "Checkmate" } else { "Stalemate or draw" }}</p>
                            <button class="btn btn-primary" on:click=move |_| {
                                let (mode, difficulty) = {
                                    let model = ctx.state.get_untracked();
                                    (model.game.game_mode, model.game.difficulty)
                                };
                                ctx.state.update(|m| m.start_game(mode, difficulty));
                            }>"Play Again"</button>
                        </div>
                    </div>
                </div>
            }.into_any()
        }}
    }
}

#[component]
fn StoryMenu() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <div class="modal-backdrop">
            <div class="panel panel-wide panel-tall">
                <div class="panel-header">
                    <button class="btn icon-btn" on:click=move |_| ctx.state.update(|m| m.set_screen(AppScreen::Menu))>"<"</button>
                    <h2>"Story Campaign"</h2>
                </div>
                <div class="panel-body stack">
                    <For
                        each=move || STORY_CHAPTERS.to_vec()
                        key=|chapter| chapter.id
                        let:chapter
                    >
                        <StoryChapterCard chapter=chapter />
                    </For>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StoryChapterCard(chapter: crate::types::StoryChapter) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <div class="option-card" style="display:grid;gap:10px;">
            <div class="split">
                <div>
                    <div class="stat-label">{format!("Chapter {}", chapter.id)}</div>
                    <h3 style="margin:0;color:#fff;">{chapter.title}</h3>
                    <div class="stat-label">{chapter.subtitle}</div>
                    <p style="margin:6px 0 0;color:#94a3b8;">{chapter.description}</p>
                </div>
                {move || {
                    let locked = ctx.state.get().campaign_progress.completed_chapters.iter().all(|id| *id + 1 != chapter.id) && chapter.id > 1;
                    if locked {
                        view! { <span class="badge">"Locked"</span> }.into_any()
                    } else {
                        view! { <button class="btn btn-indigo" on:click=move |_| ctx.state.update(|m| m.start_story(chapter.id))>"Start"</button> }.into_any()
                    }
                }}
            </div>
            <div class="row">
                <span class="badge">{format!("Enemy: {}", chapter.enemy_name)}</span>
                <span class="badge">{format!("Reward: {}", chapter.reward_skin.map(|s| s.label()).unwrap_or("None"))}</span>
            </div>
        </div>
    }
}

#[component]
fn PuzzleMenu() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <div class="modal-backdrop">
            <div class="panel">
                <div class="panel-header center-row"><h2>"Chess Puzzles"</h2></div>
                <div class="panel-body stack" style="text-align:center;">
                    <p style="color:#94a3b8;margin-top:0;">"Sharpen your tactics with local puzzle lines."</p>
                    <button class="btn btn-emerald" on:click=move |_| ctx.state.update(|m| m.start_game(GameMode::Puzzles, Difficulty::Easy))>"Start Random Puzzle"</button>
                    <div class="stack">
                        <For
                            each=move || SAMPLE_PUZZLES.to_vec()
                            key=|puzzle| puzzle.id
                            let:puzzle
                        >
                            <div class="option-card">
                                <div class="split">
                                    <span>{format!("#{} - {}", puzzle.id, puzzle.description)}</span>
                                    <span class="badge">{format!("{} Elo", puzzle.rating)}</span>
                                </div>
                            </div>
                        </For>
                    </div>
                    <div class="stat-card">
                        <div class="stat-label">"Your Rating"</div>
                        <div class="stat-value">"1200"</div>
                    </div>
                    <button class="btn btn-ghost" on:click=move |_| ctx.state.update(|m| m.set_screen(AppScreen::Menu))>"Back to Menu"</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn CustomizationMenu() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <div class="modal-backdrop">
            <div class="panel panel-wide panel-tall">
                <div class="panel-header">
                    <button class="btn icon-btn" on:click=move |_| ctx.state.update(|m| m.set_screen(AppScreen::Menu))>"<"</button>
                    <h2>"Barracks and Armory"</h2>
                </div>
                <div class="profile-layout">
                    <div class="profile-preview">
                        <div class="avatar" style=move || format!("background:{};", ctx.state.get().profile.banner_color)>
                            {move || ctx.state.get().profile.icon}
                        </div>
                        <h3>{move || ctx.state.get().player_name}</h3>
                        <div class="stat-card" style="width:100%;">
                            <div class="stat-label">"Active Loadout"</div>
                            <div class="split">
                                <span>{move || ctx.state.get().active_skin.label()}</span>
                                <span>{move || ctx.state.get().active_emote.label()}</span>
                            </div>
                        </div>
                    </div>
                    <div class="panel-body stack">
                        <h3>"Profile Identity"</h3>
                        <div class="icon-grid">
                            <For each=move || PROFILE_ICONS.to_vec() key=|icon| *icon let:icon>
                                <button class=move || {
                                    if ctx.state.get().profile.icon == icon { "identity-btn active" } else { "identity-btn" }
                                } on:click=move |_| ctx.state.update(|m| m.set_profile(|p| p.icon = icon.to_string()))>{icon}</button>
                            </For>
                        </div>
                        <div class="button-row">
                            <For each=move || BANNER_COLORS.to_vec() key=|color| *color let:color>
                                <button
                                    class="btn icon-btn"
                                    style=format!("background:{color};")
                                    on:click=move |_| ctx.state.update(|m| m.set_profile(|p| p.banner_color = color.to_string()))
                                ></button>
                            </For>
                        </div>
                        <h3>"Chess Piece Skins"</h3>
                        <div class="skin-grid">
                            <For each=move || SkinType::ALL.to_vec() key=|skin| skin.label() let:skin>
                                <button
                                    class=move || {
                                        if ctx.state.get().active_skin == skin { "skin-card active" } else { "skin-card" }
                                    }
                                    title=skin.label()
                                    on:click=move |_| ctx.state.update(|m| m.set_skin(skin))
                                >
                                    <span class="skin-card-icon">{skin_icon(skin)}</span>
                                    <span class="sr-only">{skin.label()}</span>
                                </button>
                            </For>
                        </div>
                        <h3>"Victory Emotes"</h3>
                        <div class="button-row">
                            <For each=move || ALL_EMOTES.to_vec() key=|emote| emote.label() let:emote>
                                <button
                                    class=move || {
                                        if ctx.state.get().active_emote == emote { "emote-button active" } else { "emote-button" }
                                    }
                                    title=emote.label()
                                    on:click=move |_| ctx.state.update(|m| m.set_emote(emote))
                                >
                                    <span>{emote_icon(emote)}</span>
                                    <span class="sr-only">{emote.label()}</span>
                                </button>
                            </For>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SettingsModal() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <div class="modal-backdrop">
            <div class="panel panel-wide">
                <div class="panel-header split">
                    <div class="settings-title">
                        <IconSettings />
                        <span>"Settings"</span>
                    </div>
                    <button class="btn icon-btn" on:click=move |_| ctx.state.update(|m| m.show_settings = false)>"X"</button>
                </div>
                <div class="panel-body stack">
                    <VisualSettings />
                    <button class="btn btn-indigo" on:click=move |_| ctx.state.update(|m| m.show_settings = false)>"Save Changes"</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn VisualSettings() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <h3 class="setting-section-title purple"><IconLayers />"World Theme"</h3>
        <div class="grid-4">
            <For each=move || Theme::ALL.to_vec() key=|theme| theme.label() let:theme>
                <button
                    class=move || if ctx.state.get().settings.theme == theme { "theme-card active" } else { "theme-card" }
                    style=theme_card_style(theme)
                    on:click=move |_| ctx.state.update(|m| m.update_settings(|s| s.theme = theme))
                >
                    <span class="theme-card-icon">{theme_icon(theme)}</span>
                    {theme.label()}
                </button>
            </For>
        </div>
        <h3 class="setting-section-title pink"><IconVolume />"Audio Experience"</h3>
        <div class="grid-2">
            <Toggle label="SFX and Ambience" get=move || ctx.state.get().settings.enable_sounds set=move |_| ctx.state.update(|m| m.update_settings(|s| s.enable_sounds = !s.enable_sounds)) />
        </div>
        <h3 class="setting-section-title indigo"><IconVideo />"Graphics"</h3>
        <div class="grid-3">
            <Toggle label="Animations" get=move || ctx.state.get().settings.show_animations set=move |_| ctx.state.update(|m| m.update_settings(|s| s.show_animations = !s.show_animations)) />
            <Toggle label="Particles and VFX" get=move || ctx.state.get().settings.show_vfx set=move |_| ctx.state.update(|m| m.update_settings(|s| s.show_vfx = !s.show_vfx)) />
            <Toggle label="Cinematic Kill-Cam" get=move || ctx.state.get().settings.cinematic_camera set=move |_| ctx.state.update(|m| m.update_settings(|s| s.cinematic_camera = !s.cinematic_camera)) />
        </div>
        <div class="button-row">
            <For each=move || [GraphicsQuality::Low, GraphicsQuality::Medium, GraphicsQuality::High].to_vec() key=|q| format!("{q:?}") let:q>
                <button class=move || if ctx.state.get().settings.graphics_quality == q { "btn btn-indigo" } else { "btn" }
                    on:click=move |_| ctx.state.update(|m| m.update_settings(|s| s.graphics_quality = q))>{format!("{q:?}")}</button>
            </For>
        </div>
        <h3 class="setting-section-title amber"><IconSun />"Environment"</h3>
        <div class="segmented">
            <For each=move || TimeOfDay::ALL.to_vec() key=|tod| tod.label() let:tod>
                <button class=move || if ctx.state.get().settings.time_of_day == tod { "active" } else { "" }
                    on:click=move |_| ctx.state.update(|m| m.update_settings(|s| s.time_of_day = tod))>{tod.label()}</button>
            </For>
        </div>
        <select
            class="select"
            prop:value=move || ctx.state.get().settings.weather.label()
            on:change=move |ev| {
                let value = event_target_value(&ev);
                if let Some(weather) = weather_from_label(&value) {
                    ctx.state.update(|m| m.update_settings(|s| s.weather = weather));
                }
            }
        >
            <For each=move || Weather::ALL.to_vec() key=|weather| weather.label() let:weather>
                <option value=weather.label()>{weather.label()}</option>
            </For>
        </select>
    }
}

#[component]
fn Toggle(
    label: &'static str,
    get: impl Fn() -> bool + Copy + Send + Sync + 'static,
    set: impl Fn(bool) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="option-card split" on:click=move |_| set(!get())>
            <span>{label}</span>
            <button class=move || if get() { "toggle active" } else { "toggle" }>
                <span></span>
            </button>
        </div>
    }
}

#[component]
fn AuthMenu() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let (is_login, set_is_login) = signal(true);
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    view! {
        <div class="modal-backdrop">
            <div class="panel">
                <div class="panel-header center-row">
                    <h2>{move || if is_login.get() { "Welcome Back" } else { "Create Account" }}</h2>
                </div>
                <div class="panel-body stack">
                    {move || ctx.state.get().auth_error.map(|error| view! { <div class="error-banner">{error}</div> })}
                    <form class="stack" on:submit=move |ev| {
                        ev.prevent_default();
                        let mut ok = false;
                        if is_login.get() {
                            ctx.state.update(|m| {
                                ok = m.login(&username.get(), &password.get());
                            });
                        } else {
                            ctx.state.update(|m| {
                                ok = m.register(&username.get(), &email.get(), &password.get());
                            });
                        }
                        if ok {
                            ctx.audio.speak("Signed in locally.", ctx.state.get_untracked().settings.enable_voice_overs);
                        }
                    }>
                        <input class="input" placeholder="Username" prop:value=move || username.get() on:input=move |ev| set_username.set(event_target_value(&ev)) required />
                        {move || if is_login.get() {
                            view! {}.into_any()
                        } else {
                            view! { <input class="input" placeholder="Email" prop:value=move || email.get() on:input=move |ev| set_email.set(event_target_value(&ev)) required /> }.into_any()
                        }}
                        <input class="input" placeholder="Password" type="password" prop:value=move || password.get() on:input=move |ev| set_password.set(event_target_value(&ev)) required />
                        <button class="btn btn-indigo" type="submit">{move || if is_login.get() { "Sign In" } else { "Sign Up" }}</button>
                    </form>
                    <button class="btn btn-ghost" on:click=move |_| {
                        ctx.state.update(|m| m.auth_error = None);
                        set_is_login.set(!is_login.get());
                    }>{move || if is_login.get() { "Need an account? Sign Up" } else { "Already have an account? Sign In" }}</button>
                    <button class="btn btn-ghost" on:click=move |_| ctx.state.update(|m| m.set_screen(AppScreen::Menu))>"Back to Menu"</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ProfilePage() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    view! {
        <div class="modal-backdrop">
            <div class="panel panel-wide panel-tall">
                <div class="panel-header split">
                    <h2>{move || ctx.state.get().stats().username}</h2>
                    <button class="btn icon-btn" on:click=move |_| ctx.state.update(|m| m.set_screen(AppScreen::Menu))>"X"</button>
                </div>
                <div class="panel-body stack">
                    <div class="grid-4">
                        <Stat label="ELO" value=move || ctx.state.get().stats().elo.to_string() />
                        <Stat label="Matches" value=move || ctx.state.get().stats().total_matches.to_string() />
                        <Stat label="Wins" value=move || ctx.state.get().stats().wins.to_string() />
                        <Stat label="Losses" value=move || ctx.state.get().stats().losses.to_string() />
                    </div>
                    <h3>"Recent Matches"</h3>
                    {move || {
                        let matches = ctx.state.get().matches;
                        if matches.is_empty() {
                            view! { <div class="option-card">"No matches played yet."</div> }.into_any()
                        } else {
                            matches.into_iter().map(|record| {
                                let moves = record.moves.clone();
                                view! {
                                    <div class="match-row">
                                        <div>
                                            <strong>{format!("{} vs {}", record.result.to_uppercase(), record.opponent)}</strong>
                                            <div style="color:#64748b;font-size:12px;">{record.date}</div>
                                        </div>
                                        <button class="btn btn-indigo" on:click=move |_| {
                                            let moves = moves.clone();
                                            ctx.state.update(|m| {
                                                m.game.load_replay_from_uci(&moves);
                                                m.set_screen(AppScreen::Game);
                                            });
                                        }>"Watch Replay"</button>
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn Stat(
    label: &'static str,
    value: impl Fn() -> String + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="stat-card">
            <div class="stat-label">{label}</div>
            <div class="stat-value">{move || value()}</div>
        </div>
    }
}

fn name_or_player(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "Player".to_string()
    } else {
        trimmed.to_string()
    }
}

fn piece_list(pieces: &[PieceKind]) -> String {
    if pieces.is_empty() {
        "none".to_string()
    } else {
        pieces
            .iter()
            .map(|p| p.label())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn difficulty_class(active: Difficulty, item: Difficulty) -> &'static str {
    if active == item {
        "btn btn-indigo"
    } else {
        "btn"
    }
}

fn skin_icon(skin: SkinType) -> &'static str {
    match skin {
        SkinType::Classic => "♟",
        SkinType::Marble => "◈",
        SkinType::Gold => "♛",
        SkinType::Crystal => "✦",
        SkinType::Bronze => "●",
        SkinType::Shadow => "◒",
        SkinType::Neon => "✺",
        SkinType::Wood => "◆",
    }
}

fn emote_icon(emote: crate::types::EmoteType) -> &'static str {
    match emote {
        crate::types::EmoteType::None => "∅",
        crate::types::EmoteType::Bow => "↯",
        crate::types::EmoteType::Salute => "⚑",
        crate::types::EmoteType::Spin => "↻",
        crate::types::EmoteType::Power => "⚡",
    }
}

fn theme_icon(theme: Theme) -> &'static str {
    match theme {
        Theme::Classic => "♜",
        Theme::Volcano => "▲",
        Theme::Ice => "❄",
        Theme::Cyberpunk => "◆",
        Theme::Forest => "✦",
        Theme::Castle => "♖",
        Theme::Desert => "◍",
        Theme::Galaxy => "✺",
    }
}

fn theme_card_style(theme: Theme) -> String {
    let config = theme_config(theme);
    format!(
        "--theme-bg: #{:06x}; --theme-accent: #{:06x}; --theme-light: #{:06x};",
        config.bg, config.accent, config.light
    )
}

fn weather_from_label(value: &str) -> Option<Weather> {
    Weather::ALL
        .iter()
        .copied()
        .find(|weather| weather.label() == value)
}

#[component]
fn IconLayers() -> impl IntoView {
    view! {
        <svg class="ui-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="m12 3 9 5-9 5-9-5 9-5Z" />
            <path d="m3 12 9 5 9-5" />
            <path d="m3 16 9 5 9-5" />
        </svg>
    }
}

#[component]
fn IconVolume() -> impl IntoView {
    view! {
        <svg class="ui-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M11 5 6 9H3v6h3l5 4V5Z" />
            <path d="M15.5 8.5a5 5 0 0 1 0 7" />
        </svg>
    }
}

#[component]
fn IconVideo() -> impl IntoView {
    view! {
        <svg class="ui-icon" viewBox="0 0 24 24" aria-hidden="true">
            <rect x="3" y="6" width="13" height="10" rx="2" />
            <path d="m16 10 5-3v12l-5-3v-6Z" />
        </svg>
    }
}

#[component]
fn IconSun() -> impl IntoView {
    view! {
        <svg class="ui-icon" viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="12" cy="12" r="4" />
            <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
        </svg>
    }
}
