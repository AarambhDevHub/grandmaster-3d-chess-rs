mod app;
mod audio;
mod bot;
mod constants;
mod game;
mod render;
mod storage;
mod types;
mod ui;

use app::{AppContext, AppModel};
use audio::AudioSystem;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use render::SceneCanvas;
use ui::Interface;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let state = RwSignal::new(AppModel::load());
    provide_context(AppContext {
        state,
        audio: AudioSystem,
    });

    view! {
        <main class="app">
            <SceneCanvas />
            <Interface />
        </main>
    }
}
