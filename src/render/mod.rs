use leptos::prelude::*;

#[component]
pub fn SceneCanvas() -> impl IntoView {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_scene()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        view! { <div class="canvas-fallback">"Scenix WASM scene"</div> }
    }
}

#[cfg(target_arch = "wasm32")]
fn wasm_scene() -> impl IntoView {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::app::AppContext;

    let ctx = expect_context::<AppContext>();
    let audio = ctx.audio.clone();
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
    let runtime: Rc<RefCell<Option<RenderRuntime>>> = Rc::new(RefCell::new(None));

    {
        let runtime = Rc::clone(&runtime);
        let state = ctx.state;
        Effect::new(move |_| {
            let Some(canvas) = canvas_ref.get() else {
                return;
            };
            let canvas: web_sys::HtmlCanvasElement = canvas;
            let runtime = Rc::clone(&runtime);
            let audio = audio.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(renderer) = WebGlRoyalRenderer::new(canvas.clone(), state, audio.clone())
                {
                    state.update(|model| model.renderer_error = None);
                    *runtime.borrow_mut() = Some(RenderRuntime::WebGl(renderer));
                    start_loop(Rc::clone(&runtime));
                    return;
                }

                match RoyalRenderer::new(canvas.clone(), state, audio.clone()).await {
                    Ok(renderer) => *runtime.borrow_mut() = Some(RenderRuntime::Gpu(renderer)),
                    Err(error) => {
                        *runtime.borrow_mut() = Some(RenderRuntime::Canvas(CanvasFallback::new(
                            canvas,
                            state,
                            audio,
                            js_value_text(&error),
                        )))
                    }
                }
                state.update(|model| model.renderer_error = None);
                start_loop(Rc::clone(&runtime));
            });
        });
    }

    {
        let runtime = Rc::clone(&runtime);
        let on_pointer_down = {
            let runtime = Rc::clone(&runtime);
            move |ev: leptos::ev::PointerEvent| {
                if let Some(renderer) = runtime.borrow_mut().as_mut() {
                    renderer.on_pointer_down(ev.offset_x() as f32, ev.offset_y() as f32);
                }
            }
        };
        let on_pointer_move = {
            let runtime = Rc::clone(&runtime);
            move |ev: leptos::ev::PointerEvent| {
                if let Some(renderer) = runtime.borrow_mut().as_mut() {
                    renderer.on_pointer_move(ev.offset_x() as f32, ev.offset_y() as f32);
                }
            }
        };
        let on_pointer_up = {
            let runtime = Rc::clone(&runtime);
            move |ev: leptos::ev::PointerEvent| {
                if let Some(renderer) = runtime.borrow_mut().as_mut() {
                    renderer.on_pointer_up(ev.offset_x() as f32, ev.offset_y() as f32);
                }
            }
        };
        let on_pointer_leave = {
            let runtime = Rc::clone(&runtime);
            move |ev: leptos::ev::PointerEvent| {
                if let Some(renderer) = runtime.borrow_mut().as_mut() {
                    renderer.on_pointer_up(ev.offset_x() as f32, ev.offset_y() as f32);
                }
            }
        };
        let on_wheel = {
            let runtime = Rc::clone(&runtime);
            move |ev: leptos::ev::WheelEvent| {
                ev.prevent_default();
                if let Some(renderer) = runtime.borrow_mut().as_mut() {
                    renderer.on_wheel(ev.delta_y() as f32);
                }
            }
        };
        let on_context_menu = move |ev: leptos::ev::MouseEvent| {
            ev.prevent_default();
        };
        view! {
            <canvas
                node_ref=canvas_ref
                class="scene-canvas"
                on:pointerdown=on_pointer_down
                on:pointermove=on_pointer_move
                on:pointerup=on_pointer_up
                on:pointerleave=on_pointer_leave
                on:wheel=on_wheel
                on:contextmenu=on_context_menu
            />
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_impl {
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::rc::Rc;

    use leptos::prelude::*;
    use scenix::{
        AmbientLight, Color, DirectionalLight, Easing, Geometry, LightId, MaterialId, MeshId,
        NodeAnimationTarget, NodeAnimator, PbrMaterial, PerspectiveCamera, Quat, Raycaster,
        Renderer, RendererConfig, SceneGraph, SceneNode, ScenixAnimationDriver, Transform, Vec2,
        Vec3, Vec3Track, box_geometry, cone_geometry, cylinder_geometry, sphere_geometry,
        torus_geometry, wgpu,
    };
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use web_sys::{
        CanvasRenderingContext2d, WebGlBuffer, WebGlProgram, WebGlRenderingContext as Gl,
        WebGlShader, WebGlUniformLocation,
    };

    use crate::app::AppModel;
    use crate::audio::AudioSystem;
    use crate::constants::{BOARD_SIZE, TILE_SIZE, skin_config, theme_config, weather_config};
    use crate::game::board_position;
    use crate::types::{
        BoardMutation, CameraMode, ChessPieceView, CinematicMode, GameMode, GraphicsQuality,
        PieceKind, PlayerColor, SkinType, Theme, TimeOfDay, Weather,
    };

    const TILE_LAYER: u32 = 1;
    const OBJECT_LAYER: u32 = 2;

    const MESH_TILE: MeshId = MeshId::new(1);
    const MESH_BASE: MeshId = MeshId::new(2);
    const MESH_MARKER: MeshId = MeshId::new(3);
    const MESH_BOX: MeshId = MeshId::new(4);
    const MESH_SPHERE: MeshId = MeshId::new(5);
    const MESH_CONE: MeshId = MeshId::new(6);
    const MESH_CYLINDER: MeshId = MeshId::new(7);
    const MESH_THIN_CYLINDER: MeshId = MeshId::new(8);
    const MESH_TORUS: MeshId = MeshId::new(9);
    const MESH_PARTICLE: MeshId = MeshId::new(10);

    const MAT_BASE: MaterialId = MaterialId::new(1);
    const MAT_SELECTED: MaterialId = MaterialId::new(2);
    const MAT_VALID: MaterialId = MaterialId::new(3);
    const MAT_CHECK: MaterialId = MaterialId::new(4);
    const MAT_LAST: MaterialId = MaterialId::new(5);
    const MAT_MARKER: MaterialId = MaterialId::new(6);
    const MAT_PORTAL: MaterialId = MaterialId::new(7);
    const MAT_CAPE: MaterialId = MaterialId::new(8);
    const MAT_CROWN: MaterialId = MaterialId::new(9);
    const MAT_STAFF: MaterialId = MaterialId::new(10);
    const MAT_RED_PARTICLE: MaterialId = MaterialId::new(11);
    const MAT_WHITE_PARTICLE: MaterialId = MaterialId::new(12);
    const MAT_GOLD_PARTICLE: MaterialId = MaterialId::new(13);

    pub(super) struct RoyalRenderer {
        canvas: web_sys::HtmlCanvasElement,
        renderer: Renderer,
        scene: SceneGraph,
        camera: PerspectiveCamera,
        raycaster: Raycaster,
        geometries: BTreeMap<MeshId, Geometry>,
        state: RwSignal<AppModel>,
        audio: AudioSystem,
        last_timestamp: Option<f64>,
        last_camera_key: String,
        camera_driver: ScenixAnimationDriver,
        camera_node: Option<scenix::NodeId>,
        camera_position: Vec3,
        last_move_key: String,
        move_track: Option<Vec3Track>,
        move_to_square: Option<String>,
        move_piece: Option<(PieceKind, PlayerColor)>,
        particles: Vec<Particle>,
        selected_square: Option<String>,
    }

    #[derive(Clone, Debug)]
    struct Particle {
        origin: Vec3,
        velocity: Vec3,
        age: f32,
        ttl: f32,
        material: MaterialId,
    }

    pub(super) enum RenderRuntime {
        WebGl(WebGlRoyalRenderer),
        Gpu(RoyalRenderer),
        Canvas(CanvasFallback),
    }

    impl RenderRuntime {
        fn tick(&mut self, timestamp_ms: f64) -> Result<(), JsValue> {
            match self {
                Self::WebGl(renderer) => renderer.tick(timestamp_ms),
                Self::Gpu(renderer) => renderer.tick(timestamp_ms),
                Self::Canvas(renderer) => renderer.tick(timestamp_ms),
            }
        }

        pub(super) fn on_pointer_up(&mut self, x: f32, y: f32) {
            match self {
                Self::WebGl(renderer) => renderer.on_pointer_up(x, y),
                Self::Gpu(renderer) => renderer.on_pointer_up(x, y),
                Self::Canvas(renderer) => renderer.on_pointer_up(x, y),
            }
        }

        pub(super) fn on_pointer_down(&mut self, x: f32, y: f32) {
            if let Self::WebGl(renderer) = self {
                renderer.on_pointer_down(x, y);
            }
        }

        pub(super) fn on_pointer_move(&mut self, x: f32, y: f32) {
            if let Self::WebGl(renderer) = self {
                renderer.on_pointer_move(x, y);
            }
        }

        pub(super) fn on_wheel(&mut self, delta_y: f32) {
            if let Self::WebGl(renderer) = self {
                renderer.on_wheel(delta_y);
            }
        }

        fn state(&self) -> RwSignal<AppModel> {
            match self {
                Self::WebGl(renderer) => renderer.state,
                Self::Gpu(renderer) => renderer.state,
                Self::Canvas(renderer) => renderer.state,
            }
        }
    }

    pub(super) struct WebGlRoyalRenderer {
        canvas: web_sys::HtmlCanvasElement,
        gl: Gl,
        program: WebGlProgram,
        attr_position: u32,
        attr_normal: u32,
        u_mvp: WebGlUniformLocation,
        u_model: WebGlUniformLocation,
        u_color: WebGlUniformLocation,
        u_light: WebGlUniformLocation,
        cube: GlMesh,
        cylinder: GlMesh,
        cone: GlMesh,
        state: RwSignal<AppModel>,
        audio: AudioSystem,
        last_timestamp: Option<f64>,
        phase: f32,
        width: u32,
        height: u32,
        orbit_yaw: f32,
        orbit_pitch: f32,
        orbit_distance: f32,
        dragging: bool,
        drag_distance: f32,
        last_pointer: Option<(f32, f32)>,
        last_action_key: String,
        move_timer: f32,
    }

    struct GlMesh {
        vertices: WebGlBuffer,
        indices: WebGlBuffer,
        index_count: i32,
    }

    #[derive(Clone, Copy)]
    struct CameraSpec {
        eye: [f32; 3],
        target: [f32; 3],
    }

    impl WebGlRoyalRenderer {
        pub fn new(
            canvas: web_sys::HtmlCanvasElement,
            state: RwSignal<AppModel>,
            audio: AudioSystem,
        ) -> Result<Self, JsValue> {
            let gl = webgl_context(&canvas)?;
            let program = link_program(&gl, WEBGL_VERTEX_SHADER, WEBGL_FRAGMENT_SHADER)?;
            gl.use_program(Some(&program));
            gl.enable(Gl::DEPTH_TEST);
            gl.disable(Gl::CULL_FACE);
            gl.depth_func(Gl::LEQUAL);
            gl.clear_color(0.02, 0.03, 0.06, 1.0);

            let attr_position = gl.get_attrib_location(&program, "a_position");
            let attr_normal = gl.get_attrib_location(&program, "a_normal");
            if attr_position < 0 || attr_normal < 0 {
                return Err(JsValue::from_str("webgl shader attributes missing"));
            }
            let u_mvp = uniform(&gl, &program, "u_mvp")?;
            let u_model = uniform(&gl, &program, "u_model")?;
            let u_color = uniform(&gl, &program, "u_color")?;
            let u_light = uniform(&gl, &program, "u_light")?;

            let cube = upload_mesh(&gl, &cube_mesh())?;
            let cylinder = upload_mesh(&gl, &cylinder_mesh(40, false))?;
            let cone = upload_mesh(&gl, &cylinder_mesh(40, true))?;
            let (width, height) = canvas_size(&canvas);

            Ok(Self {
                canvas,
                gl,
                program,
                attr_position: attr_position as u32,
                attr_normal: attr_normal as u32,
                u_mvp,
                u_model,
                u_color,
                u_light,
                cube,
                cylinder,
                cone,
                state,
                audio,
                last_timestamp: None,
                phase: 0.0,
                width,
                height,
                orbit_yaw: 0.0,
                orbit_pitch: 0.75,
                orbit_distance: 16.4,
                dragging: false,
                drag_distance: 0.0,
                last_pointer: None,
                last_action_key: String::new(),
                move_timer: 0.0,
            })
        }

        pub fn tick(&mut self, timestamp_ms: f64) -> Result<(), JsValue> {
            let dt = self
                .last_timestamp
                .map(|last| ((timestamp_ms - last) * 0.001).clamp(0.0, 0.08) as f32)
                .unwrap_or(1.0 / 60.0);
            self.last_timestamp = Some(timestamp_ms);
            self.phase = (self.phase + dt * 0.42) % std::f32::consts::TAU;
            self.resize();

            let model = self.state.get_untracked();
            self.update_move_timer(&model, dt);
            let aspect = self.canvas_aspect();
            let camera = self.camera_for(&model);
            let projection = perspective(50.0_f32.to_radians(), aspect, 0.1, 180.0);
            let view = look_at(camera.eye, camera.target, [0.0, 1.0, 0.0]);
            let vp = mat4_mul(projection, view);

            self.gl.use_program(Some(&self.program));
            self.gl
                .viewport(0, 0, self.width as i32, self.height as i32);
            let clear = webgl_clear_color(&model);
            self.gl.clear_color(clear[0], clear[1], clear[2], 1.0);
            self.gl.clear(Gl::COLOR_BUFFER_BIT | Gl::DEPTH_BUFFER_BIT);
            self.gl.uniform3f(Some(&self.u_light), -0.45, -1.0, -0.28);

            self.draw_stars(vp);
            self.draw_environment(vp, &model);
            self.draw_board(vp, &model);
            self.draw_pieces(vp, &model);
            self.draw_weather(vp, &model);
            self.draw_move_vfx(vp, &model);
            Ok(())
        }

        pub fn on_pointer_down(&mut self, x: f32, y: f32) {
            self.dragging = true;
            self.drag_distance = 0.0;
            self.last_pointer = Some((x, y));
        }

        pub fn on_pointer_move(&mut self, x: f32, y: f32) {
            if !self.dragging {
                return;
            }
            let Some((last_x, last_y)) = self.last_pointer else {
                self.last_pointer = Some((x, y));
                return;
            };
            let dx = x - last_x;
            let dy = y - last_y;
            self.drag_distance += dx.abs() + dy.abs();
            self.orbit_yaw -= dx * 0.006;
            self.orbit_pitch = (self.orbit_pitch + dy * 0.004).clamp(0.18, 1.34);
            self.last_pointer = Some((x, y));
            self.state
                .update(|model| model.game.camera_mode = CameraMode::Orbit);
        }

        pub fn on_pointer_up(&mut self, x: f32, y: f32) {
            let should_click = !self.dragging || self.drag_distance < 5.0;
            self.dragging = false;
            self.last_pointer = None;
            if !should_click {
                return;
            }
            let model = self.state.get_untracked();
            let camera = self.camera_for(&model);
            let css_width = self.canvas.client_width().max(1) as f32;
            let css_height = self.canvas.client_height().max(1) as f32;
            drop(model);
            if let Some(square) = pick_webgl_square(camera, css_width, css_height, x, y) {
                apply_square_click(self.state, &self.audio, &square);
            }
        }

        pub fn on_wheel(&mut self, delta_y: f32) {
            let step = if delta_y.is_sign_positive() {
                1.0
            } else {
                -1.0
            };
            self.orbit_distance = (self.orbit_distance + step * 1.1).clamp(7.0, 36.0);
            self.state
                .update(|model| model.game.camera_mode = CameraMode::Orbit);
        }

        fn camera_for(&self, model: &AppModel) -> CameraSpec {
            webgl_camera(
                model,
                self.phase,
                Some((self.orbit_yaw, self.orbit_pitch, self.orbit_distance)),
            )
        }

        fn canvas_aspect(&self) -> f32 {
            let css_width = self.canvas.client_width().max(1) as f32;
            let css_height = self.canvas.client_height().max(1) as f32;
            css_width / css_height
        }

        fn update_move_timer(&mut self, model: &AppModel, dt: f32) {
            let action_key = model
                .game
                .last_action
                .as_ref()
                .map(|action| {
                    format!(
                        "{}:{}:{:?}:{:?}",
                        action.from, action.to, action.piece, action.captured_piece
                    )
                })
                .unwrap_or_default();
            if action_key != self.last_action_key {
                self.last_action_key = action_key;
                self.move_timer = if model.game.last_action.is_some() {
                    0.95
                } else {
                    0.0
                };
            } else {
                self.move_timer = (self.move_timer - dt).max(0.0);
            }
        }

        fn resize(&mut self) {
            let ratio =
                web_sys::window().map_or(1.0, |window| window.device_pixel_ratio().max(1.0));
            let css_width = self.canvas.client_width().max(1) as f64;
            let css_height = self.canvas.client_height().max(1) as f64;
            let width = (css_width * ratio).round() as u32;
            let height = (css_height * ratio).round() as u32;
            if self.canvas.width() != width || self.canvas.height() != height {
                self.canvas.set_width(width);
                self.canvas.set_height(height);
            }
            self.width = width.max(1);
            self.height = height.max(1);
        }

        fn draw_stars(&self, vp: [f32; 16]) {
            for i in 0..96 {
                let x = ((i * 37 % 100) as f32 / 100.0 - 0.5) * 72.0;
                let y = 9.0 + (i * 53 % 100) as f32 / 100.0 * 25.0;
                let z = -32.0 - (i * 29 % 100) as f32 / 100.0 * 28.0;
                let twinkle = 0.55 + ((self.phase * 1.8 + i as f32).sin() + 1.0) * 0.18;
                self.draw_mesh(
                    &self.cube,
                    vp,
                    transform([x, y, z], [0.0, 0.0, 0.0], [0.035, 0.035, 0.035]),
                    [twinkle, twinkle, twinkle + 0.18],
                );
            }
        }

        fn draw_environment(&self, vp: [f32; 16], model: &AppModel) {
            let (_, ground, _, _, accent) = canvas_theme(model.settings.theme);
            self.draw_mesh(
                &self.cube,
                vp,
                transform([0.0, -0.56, 0.0], [0.0, 0.0, 0.0], [42.0, 0.08, 42.0]),
                time_tint(hex_rgb(ground), model.settings.time_of_day),
            );
            for i in 0..16 {
                let p = -18.0 + i as f32 * 2.4;
                self.draw_mesh(
                    &self.cube,
                    vp,
                    transform([p, -0.49, 0.0], [0.0, 0.0, 0.0], [0.018, 0.018, 36.0]),
                    scale_color(time_tint(hex_rgb(accent), model.settings.time_of_day), 0.42),
                );
                self.draw_mesh(
                    &self.cube,
                    vp,
                    transform([0.0, -0.485, p], [0.0, 0.0, 0.0], [36.0, 0.018, 0.018]),
                    scale_color(time_tint(hex_rgb(accent), model.settings.time_of_day), 0.28),
                );
            }
        }

        fn draw_board(&self, vp: [f32; 16], model: &AppModel) {
            let (_, _, light_tile, dark_tile, accent) = canvas_theme(model.settings.theme);
            self.draw_mesh(
                &self.cube,
                vp,
                transform([0.0, -0.22, 0.0], [0.0, 0.0, 0.0], [12.9, 0.42, 12.9]),
                [0.035, 0.047, 0.075],
            );

            let selected = model.game.selected_square.as_deref();
            let last_from = model.game.last_move.as_ref().map(|mv| mv.from.as_str());
            let last_to = model.game.last_move.as_ref().map(|mv| mv.to.as_str());
            for rank in 0..BOARD_SIZE {
                for file in 0..BOARD_SIZE {
                    let square = format!("{}{}", (b'a' + file as u8) as char, rank + 1);
                    let [x, z] = webgl_square_position(file, rank);
                    let mut color = if (file + rank) % 2 == 0 {
                        hex_rgb(light_tile)
                    } else {
                        hex_rgb(dark_tile)
                    };
                    if selected == Some(square.as_str()) {
                        color = [0.95, 0.72, 0.16];
                    } else if model
                        .game
                        .valid_moves
                        .iter()
                        .any(|target| target == &square)
                    {
                        color = [0.08, 0.72, 0.48];
                    } else if last_from == Some(square.as_str()) || last_to == Some(square.as_str())
                    {
                        color = hex_rgb(accent);
                    }
                    self.draw_mesh(
                        &self.cube,
                        vp,
                        transform([x, 0.0, z], [0.0, 0.0, 0.0], [1.46, 0.16, 1.46]),
                        color,
                    );
                }
            }
        }

        fn draw_pieces(&self, vp: [f32; 16], model: &AppModel) {
            let mut pieces = model.game.pieces();
            pieces.sort_by(|a, b| {
                let az = square_to_file_rank(&a.square)
                    .map(|(_, rank)| webgl_square_position(0, rank)[1])
                    .unwrap_or(0.0);
                let bz = square_to_file_rank(&b.square)
                    .map(|(_, rank)| webgl_square_position(0, rank)[1])
                    .unwrap_or(0.0);
                az.partial_cmp(&bz).unwrap_or(std::cmp::Ordering::Equal)
            });

            for piece in pieces {
                self.draw_piece(vp, model, &piece);
            }
        }

        fn draw_piece(&self, vp: [f32; 16], model: &AppModel, piece: &ChessPieceView) {
            let Some((file, rank)) = square_to_file_rank(&piece.square) else {
                return;
            };
            let [x, z] = webgl_square_position(file, rank);
            let is_last_arrival = model
                .game
                .last_action
                .as_ref()
                .is_some_and(|action| action.to == piece.square);
            let (move_lift, attack_tilt, arrival_scale) =
                if is_last_arrival && self.move_timer > 0.0 && model.settings.show_animations {
                    let progress = (1.0 - self.move_timer / 0.95).clamp(0.0, 1.0);
                    let arc = (progress * std::f32::consts::PI).sin();
                    let lift = if piece.kind == PieceKind::Knight {
                        arc * 1.15
                    } else {
                        arc * 0.52
                    };
                    let tilt = if model
                        .game
                        .last_action
                        .as_ref()
                        .is_some_and(|action| action.captured_piece.is_some())
                    {
                        (1.0 - progress) * 0.38
                    } else {
                        0.0
                    };
                    (lift, tilt, 1.0 + arc * 0.08)
                } else {
                    (0.0, 0.0, 1.0)
                };
            let skin = skin_config(model.active_skin);
            let (main, trim, accent) = if piece.color == PlayerColor::White {
                (rgb_u32(skin.white), [0.96, 0.64, 0.08], [0.92, 0.94, 0.96])
            } else {
                (
                    rgb_u32(skin.black),
                    [0.48, 0.32, 0.92],
                    [0.02, 0.025, 0.035],
                )
            };
            let y = 0.18 + move_lift;
            self.draw_mesh(
                &self.cylinder,
                vp,
                transform(
                    [x, y, z],
                    [0.0, 0.0, 0.0],
                    [0.5 * arrival_scale, 0.18, 0.5 * arrival_scale],
                ),
                trim,
            );
            self.draw_mesh(
                &self.cylinder,
                vp,
                transform(
                    [x, y + 0.34, z],
                    [attack_tilt, 0.0, 0.0],
                    [0.3 * arrival_scale, 0.68, 0.3 * arrival_scale],
                ),
                main,
            );
            match piece.kind {
                PieceKind::Pawn => {
                    self.draw_mesh(
                        &self.cone,
                        vp,
                        transform([x, y + 0.9, z], [0.0, 0.0, 0.0], [0.25, 0.2, 0.25]),
                        accent,
                    );
                    self.draw_mesh(
                        &self.cylinder,
                        vp,
                        transform([x, y + 0.76, z], [0.0, 0.0, 0.0], [0.24, 0.22, 0.24]),
                        main,
                    );
                }
                PieceKind::Knight => {
                    self.draw_mesh(
                        &self.cube,
                        vp,
                        transform(
                            [x, y + 0.78, z - 0.02],
                            [-0.38, 0.0, -0.08],
                            [0.3, 0.72, 0.4],
                        ),
                        main,
                    );
                    self.draw_mesh(
                        &self.cube,
                        vp,
                        transform([x, y + 1.1, z - 0.36], [0.0, 0.0, 0.0], [0.22, 0.24, 0.36]),
                        accent,
                    );
                }
                PieceKind::Bishop => {
                    self.draw_mesh(
                        &self.cone,
                        vp,
                        transform([x, y + 0.84, z], [0.0, 0.0, 0.0], [0.42, 0.72, 0.42]),
                        main,
                    );
                    self.draw_mesh(
                        &self.cylinder,
                        vp,
                        transform(
                            [x + 0.28, y + 0.72, z + 0.1],
                            [0.0, 0.0, 0.25],
                            [0.045, 1.18, 0.045],
                        ),
                        [0.55, 0.28, 0.05],
                    );
                    self.draw_mesh(
                        &self.cube,
                        vp,
                        transform(
                            [x + 0.36, y + 1.34, z + 0.1],
                            [0.0, 0.0, 0.75],
                            [0.12, 0.12, 0.12],
                        ),
                        [0.18, 0.75, 0.95],
                    );
                }
                PieceKind::Rook => {
                    self.draw_mesh(
                        &self.cube,
                        vp,
                        transform([x - 0.24, y + 0.46, z], [0.0, 0.0, 0.0], [0.16, 0.72, 0.26]),
                        main,
                    );
                    self.draw_mesh(
                        &self.cube,
                        vp,
                        transform([x + 0.24, y + 0.46, z], [0.0, 0.0, 0.0], [0.16, 0.72, 0.26]),
                        main,
                    );
                    self.draw_mesh(
                        &self.cube,
                        vp,
                        transform([x, y + 0.84, z], [0.0, 0.0, 0.0], [0.64, 0.36, 0.46]),
                        main,
                    );
                    for dx in [-0.22, 0.0, 0.22] {
                        self.draw_mesh(
                            &self.cube,
                            vp,
                            transform([x + dx, y + 1.12, z], [0.0, 0.0, 0.0], [0.13, 0.2, 0.58]),
                            trim,
                        );
                    }
                }
                PieceKind::Queen => {
                    self.draw_mesh(
                        &self.cone,
                        vp,
                        transform([x, y + 0.84, z], [0.0, 0.0, 0.0], [0.42, 0.82, 0.42]),
                        main,
                    );
                    self.draw_mesh(
                        &self.cylinder,
                        vp,
                        transform([x, y + 1.42, z], [1.5708, 0.0, 0.0], [0.28, 0.05, 0.28]),
                        [1.0, 0.88, 0.18],
                    );
                    self.draw_mesh(
                        &self.cylinder,
                        vp,
                        transform([x, y + 1.23, z], [0.0, 0.0, 0.0], [0.17, 0.18, 0.17]),
                        main,
                    );
                }
                PieceKind::King => {
                    self.draw_mesh(
                        &self.cube,
                        vp,
                        transform([x, y + 0.92, z], [0.0, 0.0, 0.0], [0.44, 0.86, 0.34]),
                        main,
                    );
                    self.draw_mesh(
                        &self.cube,
                        vp,
                        transform([x, y + 0.86, z - 0.24], [0.15, 0.0, 0.0], [0.4, 0.82, 0.06]),
                        [0.78, 0.06, 0.05],
                    );
                    self.draw_mesh(
                        &self.cube,
                        vp,
                        transform([x, y + 1.44, z], [0.0, 0.0, 0.0], [0.12, 0.52, 0.12]),
                        trim,
                    );
                    self.draw_mesh(
                        &self.cube,
                        vp,
                        transform([x, y + 1.55, z], [0.0, 0.0, 0.0], [0.44, 0.11, 0.11]),
                        trim,
                    );
                }
            }
        }

        fn draw_move_vfx(&self, vp: [f32; 16], model: &AppModel) {
            if self.move_timer <= 0.0 || !model.settings.show_vfx {
                return;
            }
            let Some(action) = &model.game.last_action else {
                return;
            };
            let Some((file, rank)) = square_to_file_rank(&action.to) else {
                return;
            };
            let [x, z] = webgl_square_position(file, rank);
            let progress = (1.0 - self.move_timer / 0.95).clamp(0.0, 1.0);
            let energy = 1.0 - progress;
            let count = if action.captured_piece.is_some() {
                22
            } else {
                7
            };
            let color = if action.captured_piece.is_some() {
                [0.95, 0.16, 0.12]
            } else {
                [0.95, 0.95, 1.0]
            };
            for i in 0..count {
                let angle = i as f32 * 2.399_963 + self.phase * 3.0;
                let radius = 0.28 + (i % 5) as f32 * 0.1 + progress * 1.6;
                let y = 0.42 + energy * (0.7 + (i % 4) as f32 * 0.16);
                self.draw_mesh(
                    &self.cube,
                    vp,
                    transform(
                        [x + angle.cos() * radius, y, z + angle.sin() * radius],
                        [angle, angle * 0.7, 0.0],
                        [
                            0.08 * energy.max(0.2),
                            0.08 * energy.max(0.2),
                            0.08 * energy.max(0.2),
                        ],
                    ),
                    color,
                );
            }
        }

        fn draw_weather(&self, vp: [f32; 16], model: &AppModel) {
            if model.settings.graphics_quality == GraphicsQuality::Low
                || model.settings.weather == Weather::Clear
            {
                return;
            }
            let weather = weather_config(model.settings.weather);
            let color = rgb_u32(weather.color);
            let count = weather.particle_count.min(180);
            for i in 0..count {
                let seed = i as f32;
                let x = ((i * 37 % 100) as f32 / 100.0 - 0.5) * 30.0;
                let z = ((i * 61 % 100) as f32 / 100.0 - 0.5) * 30.0;
                let fall = (self.phase * weather.speed * 8.0 + seed * 0.27).fract();
                let y = 11.0 - fall * 13.0;
                let (scale, tilt) = match model.settings.weather {
                    Weather::Rain | Weather::Storm => ([0.018, 0.42, 0.018], [0.25, 0.0, 0.0]),
                    Weather::Snow => ([0.07, 0.07, 0.07], [0.0, 0.0, 0.0]),
                    Weather::Fog => ([0.45, 0.08, 0.45], [0.0, 0.0, 0.0]),
                    Weather::Dust => ([0.09, 0.09, 0.09], [0.0, self.phase + seed, 0.0]),
                    Weather::Clear => ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
                };
                self.draw_mesh(&self.cube, vp, transform([x, y, z], tilt, scale), color);
            }
        }

        fn draw_mesh(&self, mesh: &GlMesh, vp: [f32; 16], model: [f32; 16], color: [f32; 3]) {
            let mvp = mat4_mul(vp, model);
            self.gl
                .uniform_matrix4fv_with_f32_array(Some(&self.u_mvp), false, &mvp);
            self.gl
                .uniform_matrix4fv_with_f32_array(Some(&self.u_model), false, &model);
            self.gl
                .uniform3f(Some(&self.u_color), color[0], color[1], color[2]);
            self.gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&mesh.vertices));
            self.gl.enable_vertex_attrib_array(self.attr_position);
            self.gl
                .vertex_attrib_pointer_with_i32(self.attr_position, 3, Gl::FLOAT, false, 24, 0);
            self.gl.enable_vertex_attrib_array(self.attr_normal);
            self.gl
                .vertex_attrib_pointer_with_i32(self.attr_normal, 3, Gl::FLOAT, false, 24, 12);
            self.gl
                .bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&mesh.indices));
            self.gl
                .draw_elements_with_i32(Gl::TRIANGLES, mesh.index_count, Gl::UNSIGNED_SHORT, 0);
        }
    }

    pub(super) struct CanvasFallback {
        canvas: web_sys::HtmlCanvasElement,
        state: RwSignal<AppModel>,
        audio: AudioSystem,
        init_error: String,
        last_timestamp: Option<f64>,
        phase: f64,
    }

    #[derive(Clone, Copy)]
    struct BoardProjection {
        cx: f64,
        cy: f64,
        tile_w: f64,
        tile_h: f64,
    }

    #[derive(Clone, Copy)]
    struct ScreenPoint {
        x: f64,
        y: f64,
    }

    impl CanvasFallback {
        pub fn new(
            canvas: web_sys::HtmlCanvasElement,
            state: RwSignal<AppModel>,
            audio: AudioSystem,
            init_error: String,
        ) -> Self {
            Self {
                canvas,
                state,
                audio,
                init_error,
                last_timestamp: None,
                phase: 0.0,
            }
        }

        pub fn tick(&mut self, timestamp_ms: f64) -> Result<(), JsValue> {
            let dt = self
                .last_timestamp
                .map(|last| ((timestamp_ms - last) * 0.001).clamp(0.0, 0.08))
                .unwrap_or(1.0 / 60.0);
            self.last_timestamp = Some(timestamp_ms);
            self.phase = (self.phase + dt * 0.55) % std::f64::consts::TAU;

            let context = fallback_context(&self.canvas)?;
            let ratio =
                web_sys::window().map_or(1.0, |window| window.device_pixel_ratio().max(1.0));
            let width = self.canvas.client_width().max(1) as f64;
            let height = self.canvas.client_height().max(1) as f64;
            let pixel_width = (width * ratio).round() as u32;
            let pixel_height = (height * ratio).round() as u32;
            if self.canvas.width() != pixel_width || self.canvas.height() != pixel_height {
                self.canvas.set_width(pixel_width);
                self.canvas.set_height(pixel_height);
            }
            let _ = context.set_transform(ratio, 0.0, 0.0, ratio, 0.0, 0.0);

            let model = self.state.get_untracked();
            draw_canvas_scene(
                &context,
                width,
                height,
                &model,
                self.phase,
                &self.init_error,
            );
            Ok(())
        }

        pub fn on_pointer_up(&mut self, x: f32, y: f32) {
            let width = self.canvas.client_width().max(1) as f64;
            let height = self.canvas.client_height().max(1) as f64;
            let projection = board_projection(width, height, &self.state.get_untracked());
            if let Some(square) = square_from_screen(&projection, x as f64, y as f64) {
                apply_square_click(self.state, &self.audio, &square);
            }
        }
    }

    impl RoyalRenderer {
        pub async fn new(
            canvas: web_sys::HtmlCanvasElement,
            state: RwSignal<AppModel>,
            audio: AudioSystem,
        ) -> Result<Self, JsValue> {
            scenix::set_panic_hook();
            let (width, height) = canvas_size(&canvas);
            canvas.set_width(width);
            canvas.set_height(height);
            let mut config = RendererConfig::new(width, height).vsync(true);
            config.backends = wgpu::Backends::GL;
            let mut renderer = Renderer::new(wgpu::SurfaceTarget::Canvas(canvas.clone()), config)
                .await
                .map_err(js_error)?;

            let geometries = register_assets(&mut renderer)?;
            let camera =
                PerspectiveCamera::new(50.0, width as f32 / height.max(1) as f32, 0.1, 200.0)
                    .position(Vec3::new(0.0, 12.0, 12.0))
                    .target(Vec3::ZERO);

            Ok(Self {
                canvas,
                renderer,
                scene: SceneGraph::new(),
                camera,
                raycaster: Raycaster::with_layers(TILE_LAYER),
                geometries,
                state,
                audio,
                last_timestamp: None,
                last_camera_key: String::new(),
                camera_driver: ScenixAnimationDriver::new(),
                camera_node: None,
                camera_position: Vec3::new(0.0, 12.0, 12.0),
                last_move_key: String::new(),
                move_track: None,
                move_to_square: None,
                move_piece: None,
                particles: Vec::new(),
                selected_square: None,
            })
        }

        pub fn tick(&mut self, timestamp_ms: f64) -> Result<(), JsValue> {
            let dt = self
                .last_timestamp
                .map(|last| ((timestamp_ms - last) * 0.001).max(0.0).min(0.1) as f32)
                .unwrap_or(0.0);
            self.last_timestamp = Some(timestamp_ms);
            self.resize_if_needed()?;

            let model = self.state.get_untracked();
            self.update_animation_state(&model, dt);
            self.scene = build_scene(&model, self);
            self.scene.update_world_transforms();

            self.renderer
                .render(&self.scene, &self.camera)
                .map(|_| ())
                .map_err(js_error)
        }

        pub fn on_pointer_up(&mut self, x: f32, y: f32) {
            let width = self.renderer.config().width.max(1) as f32;
            let height = self.renderer.config().height.max(1) as f32;
            let ndc = Vec2::new((x / width) * 2.0 - 1.0, 1.0 - (y / height) * 2.0);
            let ray = Raycaster::from_camera_ndc(&self.camera, ndc);
            if self
                .raycaster
                .build_bvh(&self.scene, &self.geometries)
                .is_err()
            {
                return;
            }
            let Some(hit) = self.raycaster.cast_ray(ray, &self.scene, &self.geometries) else {
                return;
            };
            let Some(node) = self.scene.get(hit.node_id) else {
                return;
            };
            let Some(square) = node.name.strip_prefix("tile:") else {
                return;
            };
            self.selected_square = Some(square.to_string());
            apply_square_click(self.state, &self.audio, square);
        }

        fn update_animation_state(&mut self, model: &AppModel, dt: f32) {
            let desired = desired_camera(model);
            let camera_key = format!(
                "{:.2}:{:.2}:{:.2}:{:?}:{:?}:{:?}",
                desired.x,
                desired.y,
                desired.z,
                model.game.turn,
                model.game.camera_mode,
                model.game.cinematic_mode
            );
            if camera_key != self.last_camera_key {
                self.last_camera_key = camera_key;
                self.camera_driver.clear();
                let mut camera_scene = SceneGraph::new();
                let node = camera_scene.add(
                    SceneNode::new("camera-proxy")
                        .transform(Transform::from_translation(self.camera_position)),
                );
                self.camera_node = Some(node);
                self.camera_driver.add_node(NodeAnimator::new(
                    node,
                    NodeAnimationTarget::Translation(Vec3Track::tween_with_easing(
                        self.camera_position,
                        desired,
                        0.75,
                        Easing::EaseOutCubic,
                    )),
                ));
            }

            if let Some(node) = self.camera_node {
                let mut temp_scene = SceneGraph::new();
                let temp_node = temp_scene.add(
                    SceneNode::new("camera-proxy")
                        .transform(Transform::from_translation(self.camera_position)),
                );
                let _ = self.camera_driver.tick(
                    dt,
                    &mut temp_scene,
                    &mut EmptyCameras,
                    &mut EmptyMaterials,
                    &mut [],
                );
                let _ = node;
                if let Some(node) = temp_scene.get(temp_node) {
                    self.camera_position = node.transform.translation;
                }
            } else {
                self.camera_position = desired;
            }
            self.camera.position = self.camera_position;
            self.camera.target = if model.game.cinematic_mode == CinematicMode::GiantKingEnding {
                Vec3::new(0.0, 3.0, 0.0)
            } else {
                Vec3::ZERO
            };

            let move_key = model
                .game
                .last_move
                .as_ref()
                .map(|mv| format!("{}:{}:{:?}", mv.from, mv.to, mv.promotion))
                .unwrap_or_default();
            if move_key != self.last_move_key {
                self.last_move_key = move_key;
                self.move_track = None;
                self.move_to_square = None;
                self.move_piece = None;
                if let Some(action) = &model.game.last_action {
                    let start = vec3_from_square(&action.from);
                    let end = vec3_from_square(&action.to);
                    let duration = if action.piece == PieceKind::Knight {
                        0.62
                    } else {
                        0.48
                    };
                    self.move_track = Some(Vec3Track::tween_with_easing(
                        start,
                        end,
                        duration,
                        Easing::EaseOutCubic,
                    ));
                    self.move_to_square = Some(action.to.clone());
                    self.move_piece = Some((action.piece, model.game.turn.opposite()));
                    if model.settings.show_vfx {
                        self.spawn_particles(
                            end,
                            if action.captured_piece.is_some() {
                                MAT_RED_PARTICLE
                            } else {
                                MAT_WHITE_PARTICLE
                            },
                            if action.captured_piece.is_some() {
                                28
                            } else {
                                8
                            },
                        );
                    }
                }
            }
            if let Some(track) = &mut self.move_track {
                track.update(dt);
                if track.is_complete() {
                    self.move_track = None;
                }
            }
            self.update_particles(dt);
        }

        fn spawn_particles(&mut self, origin: Vec3, material: MaterialId, count: usize) {
            for i in 0..count {
                let a = i as f32 * 2.399_963;
                let r = 0.04 + (i % 7) as f32 * 0.012;
                self.particles.push(Particle {
                    origin: origin + Vec3::new(a.cos() * 0.25, 0.45, a.sin() * 0.25),
                    velocity: Vec3::new(a.cos() * r, 0.08 + (i % 5) as f32 * 0.015, a.sin() * r),
                    age: 0.0,
                    ttl: 0.7 + (i % 9) as f32 * 0.035,
                    material,
                });
            }
        }

        fn update_particles(&mut self, dt: f32) {
            for particle in &mut self.particles {
                particle.age += dt;
                particle.velocity.y -= dt * 0.9;
                particle.origin += particle.velocity;
            }
            self.particles
                .retain(|particle| particle.age < particle.ttl);
        }

        fn resize_if_needed(&mut self) -> Result<(), JsValue> {
            let (width, height) = canvas_size(&self.canvas);
            if width != self.renderer.config().width || height != self.renderer.config().height {
                self.canvas.set_width(width);
                self.canvas.set_height(height);
                self.renderer.resize(width, height).map_err(js_error)?;
                self.camera.aspect = width as f32 / height.max(1) as f32;
            }
            Ok(())
        }
    }

    struct EmptyCameras;
    impl scenix::CameraStoreMut for EmptyCameras {}

    struct EmptyMaterials;
    impl scenix::PbrMaterialStoreMut for EmptyMaterials {
        fn pbr_material_mut(&mut self, _id: MaterialId) -> Option<&mut PbrMaterial> {
            None
        }
    }

    const WEBGL_VERTEX_SHADER: &str = r#"
        attribute vec3 a_position;
        attribute vec3 a_normal;
        uniform mat4 u_mvp;
        uniform mat4 u_model;
        varying vec3 v_normal;
        void main() {
            v_normal = mat3(u_model) * a_normal;
            gl_Position = u_mvp * vec4(a_position, 1.0);
        }
    "#;

    const WEBGL_FRAGMENT_SHADER: &str = r#"
        precision mediump float;
        uniform vec3 u_color;
        uniform vec3 u_light;
        varying vec3 v_normal;
        void main() {
            vec3 n = normalize(v_normal);
            float d = max(dot(n, normalize(-u_light)), 0.0);
            float rim = pow(1.0 - max(abs(n.y), 0.0), 2.0) * 0.18;
            vec3 color = u_color * (0.28 + d * 0.78) + rim;
            gl_FragColor = vec4(color, 1.0);
        }
    "#;

    fn webgl_context(canvas: &web_sys::HtmlCanvasElement) -> Result<Gl, JsValue> {
        canvas
            .get_context("webgl")?
            .or_else(|| canvas.get_context("experimental-webgl").ok().flatten())
            .ok_or_else(|| JsValue::from_str("WebGL context is unavailable"))?
            .dyn_into::<Gl>()
            .map_err(|_| JsValue::from_str("canvas context is not WebGlRenderingContext"))
    }

    fn link_program(
        gl: &Gl,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<WebGlProgram, JsValue> {
        let vertex = compile_shader(gl, Gl::VERTEX_SHADER, vertex_source)?;
        let fragment = compile_shader(gl, Gl::FRAGMENT_SHADER, fragment_source)?;
        let program = gl
            .create_program()
            .ok_or_else(|| JsValue::from_str("could not create WebGL program"))?;
        gl.attach_shader(&program, &vertex);
        gl.attach_shader(&program, &fragment);
        gl.link_program(&program);
        if gl
            .get_program_parameter(&program, Gl::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            Err(JsValue::from_str(
                &gl.get_program_info_log(&program)
                    .unwrap_or_else(|| "unknown WebGL link error".to_string()),
            ))
        }
    }

    fn compile_shader(gl: &Gl, kind: u32, source: &str) -> Result<WebGlShader, JsValue> {
        let shader = gl
            .create_shader(kind)
            .ok_or_else(|| JsValue::from_str("could not create WebGL shader"))?;
        gl.shader_source(&shader, source);
        gl.compile_shader(&shader);
        if gl
            .get_shader_parameter(&shader, Gl::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err(JsValue::from_str(
                &gl.get_shader_info_log(&shader)
                    .unwrap_or_else(|| "unknown WebGL shader error".to_string()),
            ))
        }
    }

    fn uniform(
        gl: &Gl,
        program: &WebGlProgram,
        name: &str,
    ) -> Result<WebGlUniformLocation, JsValue> {
        gl.get_uniform_location(program, name)
            .ok_or_else(|| JsValue::from_str(&format!("missing WebGL uniform {name}")))
    }

    fn upload_mesh(gl: &Gl, data: &(Vec<f32>, Vec<u16>)) -> Result<GlMesh, JsValue> {
        let vertices = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("could not create vertex buffer"))?;
        gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&vertices));
        unsafe {
            let array = js_sys::Float32Array::view(&data.0);
            gl.buffer_data_with_array_buffer_view(Gl::ARRAY_BUFFER, &array, Gl::STATIC_DRAW);
        }

        let indices = gl
            .create_buffer()
            .ok_or_else(|| JsValue::from_str("could not create index buffer"))?;
        gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&indices));
        unsafe {
            let array = js_sys::Uint16Array::view(&data.1);
            gl.buffer_data_with_array_buffer_view(
                Gl::ELEMENT_ARRAY_BUFFER,
                &array,
                Gl::STATIC_DRAW,
            );
        }

        Ok(GlMesh {
            vertices,
            indices,
            index_count: data.1.len() as i32,
        })
    }

    fn cube_mesh() -> (Vec<f32>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let faces = [
            (
                [0.0, 0.0, 1.0],
                [
                    [-0.5, -0.5, 0.5],
                    [0.5, -0.5, 0.5],
                    [0.5, 0.5, 0.5],
                    [-0.5, 0.5, 0.5],
                ],
            ),
            (
                [0.0, 0.0, -1.0],
                [
                    [0.5, -0.5, -0.5],
                    [-0.5, -0.5, -0.5],
                    [-0.5, 0.5, -0.5],
                    [0.5, 0.5, -0.5],
                ],
            ),
            (
                [1.0, 0.0, 0.0],
                [
                    [0.5, -0.5, 0.5],
                    [0.5, -0.5, -0.5],
                    [0.5, 0.5, -0.5],
                    [0.5, 0.5, 0.5],
                ],
            ),
            (
                [-1.0, 0.0, 0.0],
                [
                    [-0.5, -0.5, -0.5],
                    [-0.5, -0.5, 0.5],
                    [-0.5, 0.5, 0.5],
                    [-0.5, 0.5, -0.5],
                ],
            ),
            (
                [0.0, 1.0, 0.0],
                [
                    [-0.5, 0.5, 0.5],
                    [0.5, 0.5, 0.5],
                    [0.5, 0.5, -0.5],
                    [-0.5, 0.5, -0.5],
                ],
            ),
            (
                [0.0, -1.0, 0.0],
                [
                    [-0.5, -0.5, -0.5],
                    [0.5, -0.5, -0.5],
                    [0.5, -0.5, 0.5],
                    [-0.5, -0.5, 0.5],
                ],
            ),
        ];
        for (normal, points) in faces {
            let base = (vertices.len() / 6) as u16;
            for point in points {
                vertices.extend_from_slice(&[
                    point[0], point[1], point[2], normal[0], normal[1], normal[2],
                ]);
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        (vertices, indices)
    }

    fn cylinder_mesh(segments: usize, cone: bool) -> (Vec<f32>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let top_radius = if cone { 0.0 } else { 0.5 };
        for i in 0..segments {
            let a = i as f32 / segments as f32 * std::f32::consts::TAU;
            let x = a.cos() * 0.5;
            let z = a.sin() * 0.5;
            let normal = normalize3([a.cos(), if cone { 0.35 } else { 0.0 }, a.sin()]);
            vertices.extend_from_slice(&[x, -0.5, z, normal[0], normal[1], normal[2]]);
            vertices.extend_from_slice(&[
                a.cos() * top_radius,
                0.5,
                a.sin() * top_radius,
                normal[0],
                normal[1],
                normal[2],
            ]);
        }
        let bottom_center = (vertices.len() / 6) as u16;
        vertices.extend_from_slice(&[0.0, -0.5, 0.0, 0.0, -1.0, 0.0]);
        let top_center = (vertices.len() / 6) as u16;
        vertices.extend_from_slice(&[0.0, 0.5, 0.0, 0.0, 1.0, 0.0]);
        for i in 0..segments {
            let j = (i + 1) % segments;
            let b0 = (i * 2) as u16;
            let t0 = b0 + 1;
            let b1 = (j * 2) as u16;
            let t1 = b1 + 1;
            indices.extend_from_slice(&[b0, b1, t1, b0, t1, t0]);
            indices.extend_from_slice(&[bottom_center, b0, b1]);
            if !cone {
                indices.extend_from_slice(&[top_center, t1, t0]);
            }
        }
        (vertices, indices)
    }

    fn webgl_camera(model: &AppModel, phase: f32, orbit: Option<(f32, f32, f32)>) -> CameraSpec {
        if model.screen == crate::types::AppScreen::Menu {
            return CameraSpec {
                eye: [phase.sin() * 16.0, 9.4, phase.cos() * 16.0],
                target: [0.0, 0.45, 0.0],
            };
        }
        match model.game.camera_mode {
            CameraMode::Top => CameraSpec {
                eye: [0.0, 18.0, 0.01],
                target: [0.0, 0.0, 0.0],
            },
            CameraMode::Cinematic => CameraSpec {
                eye: [-8.0, 7.8, 8.0],
                target: [0.0, 0.15, 0.0],
            },
            CameraMode::Orbit => {
                let default_z = if model.game.game_mode == GameMode::PlayerVsPlayer
                    && model.game.turn == PlayerColor::Black
                {
                    -12.0
                } else {
                    12.0
                };
                if let Some((yaw, pitch, distance)) = orbit {
                    let y = pitch.sin() * distance;
                    let flat = pitch.cos() * distance;
                    let side = if default_z < 0.0 {
                        std::f32::consts::PI
                    } else {
                        0.0
                    };
                    let angle = yaw + side;
                    return CameraSpec {
                        eye: [angle.sin() * flat, y, angle.cos() * flat],
                        target: [0.0, 0.1, 0.0],
                    };
                }
                CameraSpec {
                    eye: [0.0, 11.2, default_z],
                    target: [0.0, 0.1, 0.0],
                }
            }
        }
    }

    fn webgl_square_position(file: usize, rank: usize) -> [f32; 2] {
        [file as f32 * 1.5 - 5.25, 5.25 - rank as f32 * 1.5]
    }

    fn pick_webgl_square(
        camera: CameraSpec,
        css_width: f32,
        css_height: f32,
        x: f32,
        y: f32,
    ) -> Option<String> {
        if css_width <= 1.0 || css_height <= 1.0 {
            return None;
        }

        let ndc_x = (x / css_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (y / css_height) * 2.0;
        let aspect = css_width / css_height;
        let tan = (50.0_f32.to_radians() * 0.5).tan();

        let forward = normalize3([
            camera.target[0] - camera.eye[0],
            camera.target[1] - camera.eye[1],
            camera.target[2] - camera.eye[2],
        ]);
        let right = normalize3(cross3(forward, [0.0, 1.0, 0.0]));
        let up = normalize3(cross3(right, forward));
        let dir = normalize3([
            forward[0] + right[0] * ndc_x * tan * aspect + up[0] * ndc_y * tan,
            forward[1] + right[1] * ndc_x * tan * aspect + up[1] * ndc_y * tan,
            forward[2] + right[2] * ndc_x * tan * aspect + up[2] * ndc_y * tan,
        ]);

        if dir[1].abs() < 0.0001 {
            return None;
        }
        let plane_y = 0.1;
        let t = (plane_y - camera.eye[1]) / dir[1];
        if t <= 0.0 {
            return None;
        }
        let hit_x = camera.eye[0] + dir[0] * t;
        let hit_z = camera.eye[2] + dir[2] * t;
        if !(-6.0..=6.0).contains(&hit_x) || !(-6.0..=6.0).contains(&hit_z) {
            return None;
        }

        let file = ((hit_x + 6.0) / 1.5).floor() as i32;
        let rank = ((6.0 - hit_z) / 1.5).floor() as i32;
        if !(0..8).contains(&file) || !(0..8).contains(&rank) {
            return None;
        }
        Some(format!("{}{}", (b'a' + file as u8) as char, rank + 1))
    }

    fn transform(translation: [f32; 3], rotation: [f32; 3], scale: [f32; 3]) -> [f32; 16] {
        let t = translation_mat4(translation);
        let rx = rotation_x(rotation[0]);
        let ry = rotation_y(rotation[1]);
        let rz = rotation_z(rotation[2]);
        let s = scale_mat4(scale);
        mat4_mul(t, mat4_mul(ry, mat4_mul(rx, mat4_mul(rz, s))))
    }

    fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
        let f = 1.0 / (fovy * 0.5).tan();
        let nf = 1.0 / (near - far);
        [
            f / aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            f,
            0.0,
            0.0,
            0.0,
            0.0,
            (far + near) * nf,
            -1.0,
            0.0,
            0.0,
            2.0 * far * near * nf,
            0.0,
        ]
    }

    fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [f32; 16] {
        let z = normalize3([eye[0] - target[0], eye[1] - target[1], eye[2] - target[2]]);
        let x = normalize3(cross3(up, z));
        let y = cross3(z, x);
        [
            x[0],
            y[0],
            z[0],
            0.0,
            x[1],
            y[1],
            z[1],
            0.0,
            x[2],
            y[2],
            z[2],
            0.0,
            -dot3(x, eye),
            -dot3(y, eye),
            -dot3(z, eye),
            1.0,
        ]
    }

    fn mat4_mul(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
        let mut out = [0.0; 16];
        for col in 0..4 {
            for row in 0..4 {
                out[col * 4 + row] = a[row] * b[col * 4]
                    + a[4 + row] * b[col * 4 + 1]
                    + a[8 + row] * b[col * 4 + 2]
                    + a[12 + row] * b[col * 4 + 3];
            }
        }
        out
    }

    fn translation_mat4(v: [f32; 3]) -> [f32; 16] {
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, v[0], v[1], v[2], 1.0,
        ]
    }

    fn scale_mat4(v: [f32; 3]) -> [f32; 16] {
        [
            v[0], 0.0, 0.0, 0.0, 0.0, v[1], 0.0, 0.0, 0.0, 0.0, v[2], 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn rotation_x(a: f32) -> [f32; 16] {
        let (s, c) = a.sin_cos();
        [
            1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn rotation_y(a: f32) -> [f32; 16] {
        let (s, c) = a.sin_cos();
        [
            c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn rotation_z(a: f32) -> [f32; 16] {
        let (s, c) = a.sin_cos();
        [
            c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn normalize3(v: [f32; 3]) -> [f32; 3] {
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(0.0001);
        [v[0] / len, v[1] / len, v[2] / len]
    }

    fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    fn hex_rgb(hex: &str) -> [f32; 3] {
        let clean = hex.trim_start_matches('#');
        if clean.len() < 6 {
            return [0.4, 0.45, 0.52];
        }
        let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(100) as f32 / 255.0;
        let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(110) as f32 / 255.0;
        let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(125) as f32 / 255.0;
        [r, g, b]
    }

    fn webgl_clear_color(model: &AppModel) -> [f32; 3] {
        let (bg, _, _, _, _) = canvas_theme(model.settings.theme);
        time_tint(hex_rgb(bg), model.settings.time_of_day)
    }

    fn time_tint(color: [f32; 3], time_of_day: TimeOfDay) -> [f32; 3] {
        match time_of_day {
            TimeOfDay::Morning => mix_color(scale_color(color, 0.85), [0.98, 0.72, 0.42], 0.08),
            TimeOfDay::Noon => color,
            TimeOfDay::Sunset => mix_color(scale_color(color, 0.75), [0.95, 0.35, 0.14], 0.14),
            TimeOfDay::Night => scale_color(color, 0.46),
        }
    }

    fn mix_color(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
        [
            a[0] * (1.0 - t) + b[0] * t,
            a[1] * (1.0 - t) + b[1] * t,
            a[2] * (1.0 - t) + b[2] * t,
        ]
    }

    fn rgb_u32(hex: u32) -> [f32; 3] {
        [
            ((hex >> 16) & 0xff) as f32 / 255.0,
            ((hex >> 8) & 0xff) as f32 / 255.0,
            (hex & 0xff) as f32 / 255.0,
        ]
    }

    fn scale_color(color: [f32; 3], scale: f32) -> [f32; 3] {
        [color[0] * scale, color[1] * scale, color[2] * scale]
    }

    fn apply_square_click(state: RwSignal<AppModel>, audio: &AudioSystem, square: &str) {
        let mut move_sound = None;
        state.update(|model| {
            let before_capture_count =
                model.game.captured.white.len() + model.game.captured.black.len();
            let moved = model.game.click_square(square);
            if moved {
                let after_capture_count =
                    model.game.captured.white.len() + model.game.captured.black.len();
                model.after_move();
                move_sound = Some((
                    model.settings.enable_sounds,
                    after_capture_count > before_capture_count,
                    model.game.is_game_over,
                ));
            }
        });
        if let Some((enabled, captured, game_over)) = move_sound {
            if game_over {
                audio.play_win(enabled);
            } else if captured {
                audio.play_capture(enabled);
            } else {
                audio.play_move(enabled);
            }
        }
    }

    fn fallback_context(
        canvas: &web_sys::HtmlCanvasElement,
    ) -> Result<CanvasRenderingContext2d, JsValue> {
        canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("2D canvas context is unavailable"))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| JsValue::from_str("canvas context is not CanvasRenderingContext2d"))
    }

    fn draw_canvas_scene(
        context: &CanvasRenderingContext2d,
        width: f64,
        height: f64,
        model: &AppModel,
        phase: f64,
        init_error: &str,
    ) {
        let (background, ground, light_tile, dark_tile, accent) =
            canvas_theme(model.settings.theme);
        context.set_fill_style_str(background);
        context.fill_rect(0.0, 0.0, width, height);

        draw_canvas_environment(context, width, height, ground, accent, phase, model);

        let projection = board_projection(width, height, model);
        draw_canvas_board(context, &projection, model, light_tile, dark_tile, accent);
        draw_canvas_pieces(context, &projection, model, phase);
        draw_canvas_vfx(context, &projection, model, accent, phase);

        context.set_font("700 11px Inter, sans-serif");
        context.set_fill_style_str("rgba(226, 232, 240, 0.58)");
        let _ = context.fill_text("SCENIX GPU FALLBACK - ROYAL CANVAS 3D", 24.0, height - 28.0);
        context.set_font("500 10px Inter, sans-serif");
        context.set_fill_style_str("rgba(148, 163, 184, 0.48)");
        let _ = context.fill_text(init_error, 24.0, height - 12.0);
    }

    fn draw_canvas_environment(
        context: &CanvasRenderingContext2d,
        width: f64,
        height: f64,
        ground: &str,
        accent: &str,
        phase: f64,
        model: &AppModel,
    ) {
        context.set_global_alpha(0.28);
        context.set_fill_style_str(accent);
        context.begin_path();
        let _ = context.arc(
            width * (0.64 + phase.sin() * 0.03),
            height * 0.28,
            height * 0.22,
            0.0,
            std::f64::consts::TAU,
        );
        context.fill();
        context.set_global_alpha(1.0);

        context.set_fill_style_str(ground);
        context.fill_rect(0.0, height * 0.58, width, height * 0.42);

        context.set_stroke_style_str("rgba(148, 163, 184, 0.13)");
        context.set_line_width(1.0);
        let horizon = height * 0.58;
        for i in 0..22 {
            let x = width * (i as f64 / 21.0);
            context.begin_path();
            context.move_to(width * 0.5, horizon);
            context.line_to(x, height - 34.0);
            context.stroke();
        }
        for i in 0..12 {
            let t = i as f64 / 11.0;
            let y = horizon + (height - 34.0 - horizon) * t * t;
            context.begin_path();
            context.move_to(24.0, y);
            context.line_to(width - 24.0, y);
            context.stroke();
        }

        if model.settings.show_vfx {
            context.set_fill_style_str("rgba(250, 204, 21, 0.62)");
            for i in 0..18 {
                let a = phase * 0.8 + i as f64 * 2.399;
                let r = 120.0 + (i % 6) as f64 * 42.0;
                context.begin_path();
                let _ = context.arc(
                    width * 0.5 + a.cos() * r,
                    height * 0.34 + a.sin() * r * 0.28,
                    1.8 + (i % 3) as f64,
                    0.0,
                    std::f64::consts::TAU,
                );
                context.fill();
            }
        }
    }

    fn draw_canvas_board(
        context: &CanvasRenderingContext2d,
        projection: &BoardProjection,
        model: &AppModel,
        light_tile: &str,
        dark_tile: &str,
        accent: &str,
    ) {
        context.set_global_alpha(0.42);
        context.set_fill_style_str("#020617");
        draw_diamond(
            context,
            projection.cx,
            projection.cy + projection.tile_h * 0.65,
            projection.tile_w * 8.75,
            projection.tile_h * 8.75,
        );
        context.set_global_alpha(1.0);

        let selected = model.game.selected_square.as_deref();
        let last_from = model.game.last_move.as_ref().map(|mv| mv.from.as_str());
        let last_to = model.game.last_move.as_ref().map(|mv| mv.to.as_str());
        let check_square = if model.game.is_check {
            model
                .game
                .pieces()
                .into_iter()
                .find(|piece| piece.kind == PieceKind::King && piece.color == model.game.turn)
                .map(|piece| piece.square)
        } else {
            None
        };

        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                let square = format!("{}{}", (b'a' + file as u8) as char, rank + 1);
                let point = project_square(projection, file as f64, rank as f64);
                let is_dark = (file + rank) % 2 == 1;
                context.set_fill_style_str(if is_dark { dark_tile } else { light_tile });
                draw_diamond(
                    context,
                    point.x,
                    point.y,
                    projection.tile_w,
                    projection.tile_h,
                );

                let highlight = if selected == Some(square.as_str()) {
                    Some("rgba(250, 204, 21, 0.48)")
                } else if model
                    .game
                    .valid_moves
                    .iter()
                    .any(|target| target == &square)
                {
                    Some("rgba(52, 211, 153, 0.34)")
                } else if last_from == Some(square.as_str()) || last_to == Some(square.as_str()) {
                    Some("rgba(129, 140, 248, 0.34)")
                } else if check_square.as_deref() == Some(square.as_str()) {
                    Some("rgba(239, 68, 68, 0.46)")
                } else {
                    None
                };
                if let Some(color) = highlight {
                    context.set_fill_style_str(color);
                    draw_diamond(
                        context,
                        point.x,
                        point.y,
                        projection.tile_w,
                        projection.tile_h,
                    );
                }

                context.set_stroke_style_str("rgba(15, 23, 42, 0.4)");
                context.set_line_width(1.0);
                stroke_diamond(
                    context,
                    point.x,
                    point.y,
                    projection.tile_w,
                    projection.tile_h,
                );
            }
        }

        context.set_stroke_style_str(accent);
        context.set_global_alpha(0.66);
        context.set_line_width(2.0);
        stroke_diamond(
            context,
            projection.cx,
            projection.cy,
            projection.tile_w * 8.08,
            projection.tile_h * 8.08,
        );
        context.set_global_alpha(1.0);
    }

    fn draw_canvas_pieces(
        context: &CanvasRenderingContext2d,
        projection: &BoardProjection,
        model: &AppModel,
        phase: f64,
    ) {
        let mut pieces = model.game.pieces();
        pieces.sort_by(|a, b| {
            let ay = square_screen_y(projection, &a.square);
            let by = square_screen_y(projection, &b.square);
            ay.partial_cmp(&by).unwrap_or(std::cmp::Ordering::Equal)
        });

        for piece in pieces {
            draw_canvas_piece(context, projection, &piece, model, phase);
        }
    }

    fn draw_canvas_piece(
        context: &CanvasRenderingContext2d,
        projection: &BoardProjection,
        piece: &ChessPieceView,
        model: &AppModel,
        phase: f64,
    ) {
        let Some((file, rank)) = square_to_file_rank(&piece.square) else {
            return;
        };
        let mut point = project_square(projection, file as f64, rank as f64);
        if model
            .game
            .last_action
            .as_ref()
            .is_some_and(|action| action.to == piece.square)
        {
            point.y += phase.sin() * 2.8;
        }
        let scale = projection.tile_w / 100.0;
        let height = match piece.kind {
            PieceKind::Pawn => 52.0,
            PieceKind::Knight | PieceKind::Bishop => 68.0,
            PieceKind::Rook => 70.0,
            PieceKind::Queen => 82.0,
            PieceKind::King => 88.0,
        } * scale;
        let base_w = projection.tile_w * 0.48;
        let body_w = base_w * 0.64;
        let top_y = point.y - height;
        let light = piece.color == PlayerColor::White;
        let fill = if light { "#f8fafc" } else { "#1e293b" };
        let side = if light { "#cbd5e1" } else { "#020617" };
        let stroke = if light { "#fbbf24" } else { "#8b5cf6" };

        context.set_global_alpha(0.34);
        context.set_fill_style_str("#000000");
        context.begin_path();
        let _ = context.ellipse(
            point.x,
            point.y + projection.tile_h * 0.17,
            base_w * 0.58,
            projection.tile_h * 0.18,
            0.0,
            0.0,
            std::f64::consts::TAU,
        );
        context.fill();
        context.set_global_alpha(1.0);

        context.set_fill_style_str(side);
        context.fill_rect(
            point.x - body_w * 0.42,
            top_y + height * 0.28,
            body_w * 0.84,
            height * 0.58,
        );
        context.set_fill_style_str(fill);
        context.begin_path();
        context.move_to(point.x - body_w * 0.5, point.y - 8.0 * scale);
        context.line_to(point.x - body_w * 0.28, top_y + height * 0.26);
        context.line_to(point.x + body_w * 0.28, top_y + height * 0.26);
        context.line_to(point.x + body_w * 0.5, point.y - 8.0 * scale);
        context.close_path();
        context.fill();

        context.set_fill_style_str(fill);
        context.begin_path();
        let _ = context.ellipse(
            point.x,
            point.y - 7.0 * scale,
            base_w * 0.48,
            projection.tile_h * 0.18,
            0.0,
            0.0,
            std::f64::consts::TAU,
        );
        context.fill();
        context.set_stroke_style_str(stroke);
        context.set_line_width(2.0);
        context.stroke();

        draw_piece_head(context, point.x, top_y, scale, piece.kind, fill, stroke);

        context.set_font(&format!(
            "800 {}px Georgia, serif",
            (22.0 * scale).max(15.0)
        ));
        context.set_text_align("center");
        context.set_fill_style_str(if light { "#0f172a" } else { "#f8fafc" });
        let _ = context.fill_text(piece.kind.label(), point.x, point.y - height * 0.38);
        context.set_text_align("start");
    }

    fn draw_piece_head(
        context: &CanvasRenderingContext2d,
        x: f64,
        top_y: f64,
        scale: f64,
        kind: PieceKind,
        fill: &str,
        stroke: &str,
    ) {
        context.set_fill_style_str(fill);
        context.set_stroke_style_str(stroke);
        context.set_line_width(2.0);
        match kind {
            PieceKind::King => {
                context.fill_rect(
                    x - 5.0 * scale,
                    top_y - 18.0 * scale,
                    10.0 * scale,
                    28.0 * scale,
                );
                context.fill_rect(
                    x - 16.0 * scale,
                    top_y - 8.0 * scale,
                    32.0 * scale,
                    8.0 * scale,
                );
            }
            PieceKind::Queen => {
                context.begin_path();
                context.move_to(x - 24.0 * scale, top_y + 20.0 * scale);
                context.line_to(x - 16.0 * scale, top_y - 12.0 * scale);
                context.line_to(x, top_y + 8.0 * scale);
                context.line_to(x + 16.0 * scale, top_y - 12.0 * scale);
                context.line_to(x + 24.0 * scale, top_y + 20.0 * scale);
                context.close_path();
                context.fill();
                context.stroke();
            }
            PieceKind::Rook => {
                context.fill_rect(
                    x - 22.0 * scale,
                    top_y + 2.0 * scale,
                    44.0 * scale,
                    24.0 * scale,
                );
                for i in [-1.0, 0.0, 1.0] {
                    context.clear_rect(
                        x + i * 14.0 * scale - 4.0 * scale,
                        top_y + 2.0 * scale,
                        8.0 * scale,
                        8.0 * scale,
                    );
                }
            }
            PieceKind::Bishop => {
                context.begin_path();
                let _ = context.ellipse(
                    x,
                    top_y + 10.0 * scale,
                    19.0 * scale,
                    25.0 * scale,
                    0.0,
                    0.0,
                    std::f64::consts::TAU,
                );
                context.fill();
                context.stroke();
            }
            PieceKind::Knight => {
                context.begin_path();
                context.move_to(x - 18.0 * scale, top_y + 26.0 * scale);
                context.line_to(x - 6.0 * scale, top_y - 12.0 * scale);
                context.line_to(x + 20.0 * scale, top_y + 4.0 * scale);
                context.line_to(x + 8.0 * scale, top_y + 28.0 * scale);
                context.close_path();
                context.fill();
                context.stroke();
            }
            PieceKind::Pawn => {
                context.begin_path();
                let _ = context.arc(
                    x,
                    top_y + 14.0 * scale,
                    17.0 * scale,
                    0.0,
                    std::f64::consts::TAU,
                );
                context.fill();
                context.stroke();
            }
        }
    }

    fn draw_canvas_vfx(
        context: &CanvasRenderingContext2d,
        projection: &BoardProjection,
        model: &AppModel,
        accent: &str,
        phase: f64,
    ) {
        if !model.settings.show_vfx {
            return;
        }
        if let Some(action) = &model.game.last_action {
            if let Some((file, rank)) = square_to_file_rank(&action.to) {
                let point = project_square(projection, file as f64, rank as f64);
                context.set_global_alpha(0.72);
                context.set_stroke_style_str(if action.captured_piece.is_some() {
                    "#ef4444"
                } else {
                    accent
                });
                context.set_line_width(2.0);
                for i in 0..3 {
                    context.begin_path();
                    let radius = projection.tile_w * (0.24 + i as f64 * 0.08) + phase.sin() * 2.0;
                    let _ = context.arc(
                        point.x,
                        point.y - projection.tile_h * 0.18,
                        radius,
                        0.0,
                        std::f64::consts::TAU,
                    );
                    context.stroke();
                }
                context.set_global_alpha(1.0);
            }
        }
    }

    fn draw_diamond(context: &CanvasRenderingContext2d, x: f64, y: f64, width: f64, height: f64) {
        context.begin_path();
        context.move_to(x, y - height * 0.5);
        context.line_to(x + width * 0.5, y);
        context.line_to(x, y + height * 0.5);
        context.line_to(x - width * 0.5, y);
        context.close_path();
        context.fill();
    }

    fn stroke_diamond(context: &CanvasRenderingContext2d, x: f64, y: f64, width: f64, height: f64) {
        context.begin_path();
        context.move_to(x, y - height * 0.5);
        context.line_to(x + width * 0.5, y);
        context.line_to(x, y + height * 0.5);
        context.line_to(x - width * 0.5, y);
        context.close_path();
        context.stroke();
    }

    fn board_projection(width: f64, height: f64, model: &AppModel) -> BoardProjection {
        let base = width.min(height);
        let tile_w = (base * 0.126).clamp(58.0, 142.0);
        let tile_h = tile_w * 0.52;
        let cy = if model.screen == crate::types::AppScreen::Game {
            height * 0.46
        } else {
            height * 0.42
        };
        BoardProjection {
            cx: width * 0.5,
            cy,
            tile_w,
            tile_h,
        }
    }

    fn project_square(projection: &BoardProjection, file: f64, rank: f64) -> ScreenPoint {
        let f = file - 3.5;
        let r = rank - 3.5;
        ScreenPoint {
            x: projection.cx + (f - r) * projection.tile_w * 0.5,
            y: projection.cy + (f + r) * projection.tile_h * 0.5,
        }
    }

    fn square_from_screen(projection: &BoardProjection, x: f64, y: f64) -> Option<String> {
        let dx = (x - projection.cx) / (projection.tile_w * 0.5);
        let dy = (y - projection.cy) / (projection.tile_h * 0.5);
        let file = ((dy + dx) * 0.5 + 3.5).round() as i32;
        let rank = ((dy - dx) * 0.5 + 3.5).round() as i32;
        if (0..8).contains(&file) && (0..8).contains(&rank) {
            Some(format!("{}{}", (b'a' + file as u8) as char, rank + 1))
        } else {
            None
        }
    }

    fn square_screen_y(projection: &BoardProjection, square: &str) -> f64 {
        square_to_file_rank(square)
            .map(|(file, rank)| project_square(projection, file as f64, rank as f64).y)
            .unwrap_or(0.0)
    }

    fn square_to_file_rank(square: &str) -> Option<(usize, usize)> {
        let bytes = square.as_bytes();
        if bytes.len() < 2 {
            return None;
        }
        let file = bytes[0].to_ascii_lowercase().checked_sub(b'a')? as usize;
        let rank = bytes[1].checked_sub(b'1')? as usize;
        if file < 8 && rank < 8 {
            Some((file, rank))
        } else {
            None
        }
    }

    fn canvas_theme(
        theme: Theme,
    ) -> (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ) {
        match theme {
            Theme::Classic => ("#080d18", "#101827", "#d6d3c5", "#455064", "#facc15"),
            Theme::Volcano => ("#130806", "#24100d", "#f59e0b", "#7f1d1d", "#fb7185"),
            Theme::Ice => ("#06121e", "#0f2738", "#dff7ff", "#5b8fb1", "#67e8f9"),
            Theme::Cyberpunk => ("#050716", "#0c1028", "#8b5cf6", "#111827", "#22d3ee"),
            Theme::Forest => ("#06140d", "#0f2416", "#bad7a0", "#36543a", "#86efac"),
            Theme::Castle => ("#0d111d", "#171c2a", "#cbd5e1", "#475569", "#fbbf24"),
            Theme::Desert => ("#1a1207", "#2a1b0c", "#f5deb3", "#9a6a35", "#fde68a"),
            Theme::Galaxy => ("#050312", "#111032", "#c4b5fd", "#312e81", "#a78bfa"),
        }
    }

    fn build_scene(model: &AppModel, runtime: &RoyalRenderer) -> SceneGraph {
        let mut scene = SceneGraph::new();
        let theme = theme_config(model.settings.theme);
        add_lights(&mut scene, model.settings.theme, theme.ambient);
        add_environment(&mut scene, model, runtime);
        add_board(&mut scene, model, runtime);
        add_pieces(&mut scene, model, runtime);
        add_particles(&mut scene, runtime);
        scene
    }

    fn add_lights(scene: &mut SceneGraph, theme: Theme, ambient: f32) {
        scene.add(SceneNode::light("ambient", LightId::new(1)));
        scene.add(
            SceneNode::light("sun", LightId::new(2)).transform(Transform::from_translation(
                match theme {
                    Theme::Volcano => Vec3::new(-6.0, 9.0, 4.0),
                    Theme::Cyberpunk => Vec3::new(4.0, 8.0, -6.0),
                    _ => Vec3::new(6.0, 12.0, 8.0),
                },
            )),
        );
        let _ = ambient;
    }

    fn add_environment(scene: &mut SceneGraph, model: &AppModel, runtime: &RoyalRenderer) {
        let ground_scale = match model.settings.theme {
            Theme::Cyberpunk => Vec3::new(40.0, 0.04, 40.0),
            Theme::Desert => Vec3::new(60.0, 0.04, 60.0),
            _ => Vec3::new(32.0, 0.04, 32.0),
        };
        scene.add(
            SceneNode::mesh(
                "environment-ground",
                MESH_BOX,
                theme_bg_material(model.settings.theme),
            )
            .transform(Transform::new(
                Vec3::new(0.0, -0.72, 0.0),
                Quat::IDENTITY,
                ground_scale,
            ))
            .layer(OBJECT_LAYER),
        );

        if model.settings.theme == Theme::Cyberpunk {
            for i in 0..10 {
                let x = -18.0 + i as f32 * 4.0;
                scene.add(
                    SceneNode::mesh("cyber-grid", MESH_THIN_CYLINDER, MAT_PORTAL)
                        .transform(Transform::new(
                            Vec3::new(x, 0.05, -18.0),
                            Quat::from_axis_angle(Vec3::X, std::f32::consts::FRAC_PI_2),
                            Vec3::new(1.0, 1.0, 16.0),
                        ))
                        .layer(OBJECT_LAYER),
                );
                scene.add(
                    SceneNode::mesh("cyber-grid", MESH_THIN_CYLINDER, MAT_PORTAL)
                        .transform(Transform::new(
                            Vec3::new(-18.0, 0.05, x),
                            Quat::from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_2),
                            Vec3::new(1.0, 1.0, 16.0),
                        ))
                        .layer(OBJECT_LAYER),
                );
            }
        }

        let weather = weather_config(model.settings.weather);
        if weather.particle_count > 0 && model.settings.graphics_quality != GraphicsQuality::Low {
            for i in 0..weather.particle_count {
                let x = ((i * 37) % 100) as f32 / 100.0 * 38.0 - 19.0;
                let z = ((i * 61) % 100) as f32 / 100.0 * 38.0 - 19.0;
                let y = 3.0 + ((i * 17) % 100) as f32 / 100.0 * 13.0;
                let fall = (runtime.last_timestamp.unwrap_or(0.0) as f32 * 0.001 * weather.speed
                    + i as f32 * 0.013)
                    % 8.0;
                scene.add(
                    SceneNode::mesh(
                        "weather",
                        MESH_PARTICLE,
                        weather_material(model.settings.weather),
                    )
                    .transform(Transform::new(
                        Vec3::new(x, y - fall, z),
                        Quat::IDENTITY,
                        splat(if model.settings.weather == crate::types::Weather::Fog {
                            0.16
                        } else {
                            0.045
                        }),
                    ))
                    .layer(OBJECT_LAYER),
                );
            }
        }
    }

    fn add_board(scene: &mut SceneGraph, model: &AppModel, _runtime: &RoyalRenderer) {
        let mutation_scale = if model.settings.board_mutation == BoardMutation::Giant {
            1.12
        } else {
            1.0
        };
        scene.add(
            SceneNode::mesh("board-base", MESH_BASE, MAT_BASE)
                .transform(Transform::new(
                    Vec3::new(0.0, -0.25, 0.0),
                    Quat::IDENTITY,
                    splat(mutation_scale),
                ))
                .layer(OBJECT_LAYER),
        );

        let selected = model.game.selected_square.as_deref();
        let valid = &model.game.valid_moves;
        let last_from = model.game.last_move.as_ref().map(|mv| mv.from.as_str());
        let last_to = model.game.last_move.as_ref().map(|mv| mv.to.as_str());
        let check_square = if model.game.is_check {
            model
                .game
                .pieces()
                .into_iter()
                .find(|piece| piece.kind == PieceKind::King && piece.color == model.game.turn)
                .map(|piece| piece.square)
        } else {
            None
        };

        for rank in 0..BOARD_SIZE {
            for file in 0..BOARD_SIZE {
                let square = format!("{}{}", (b'a' + file as u8) as char, rank + 1);
                let (x, _, z) = board_position(&square);
                let material = if selected == Some(square.as_str()) {
                    MAT_SELECTED
                } else if check_square.as_deref() == Some(square.as_str()) {
                    MAT_CHECK
                } else if valid.iter().any(|mv| mv == &square) {
                    MAT_VALID
                } else if last_from == Some(square.as_str()) || last_to == Some(square.as_str()) {
                    MAT_LAST
                } else {
                    theme_tile_material(model.settings.theme, (rank + file) % 2 != 0)
                };
                scene.add(
                    SceneNode::mesh(format!("tile:{square}"), MESH_TILE, material)
                        .transform(Transform::new(
                            Vec3::new(x, 0.0, z),
                            Quat::IDENTITY,
                            splat(mutation_scale),
                        ))
                        .layer(TILE_LAYER),
                );

                if valid.iter().any(|mv| mv == &square)
                    && !model
                        .game
                        .pieces()
                        .iter()
                        .any(|piece| piece.square == square)
                {
                    scene.add(
                        SceneNode::mesh("valid-marker", MESH_MARKER, MAT_MARKER)
                            .transform(Transform::from_translation(Vec3::new(x, 0.3, z)))
                            .layer(OBJECT_LAYER),
                    );
                }
                if model.settings.board_mutation == BoardMutation::Portals
                    && (square == "a1" || square == "h8")
                {
                    scene.add(
                        SceneNode::mesh("portal", MESH_MARKER, MAT_PORTAL)
                            .transform(Transform::new(
                                Vec3::new(x, 0.34, z),
                                Quat::IDENTITY,
                                Vec3::new(2.0, 0.25, 2.0),
                            ))
                            .layer(OBJECT_LAYER),
                    );
                }
            }
        }
    }

    fn add_pieces(scene: &mut SceneGraph, model: &AppModel, runtime: &RoyalRenderer) {
        for piece in model.game.pieces() {
            let mut pos = vec3_from_square(&piece.square);
            if let (Some(to), Some((kind, color)), Some(track)) = (
                runtime.move_to_square.as_ref(),
                runtime.move_piece,
                runtime.move_track.as_ref(),
            ) {
                if to == &piece.square && kind == piece.kind && color == piece.color {
                    pos = track.value();
                    if piece.kind == PieceKind::Knight {
                        let p = track.value();
                        let lift = (track.value().x - vec3_from_square(to).x).abs().min(1.0);
                        pos = Vec3::new(p.x, p.y + 1.2 * lift, p.z);
                    }
                }
            }
            add_piece(scene, piece, pos, model);
        }
    }

    fn add_piece(scene: &mut SceneGraph, piece: ChessPieceView, pos: Vec3, model: &AppModel) {
        let mat = skin_material(model.active_skin, piece.color);
        let rot = if piece.color == PlayerColor::Black {
            Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI)
        } else {
            Quat::IDENTITY
        };
        let mut scale = Vec3::ONE;
        if model.game.cinematic_mode == CinematicMode::GiantKingEnding {
            if piece.kind == PieceKind::King && Some(piece.color) == model.game.winner {
                scale = splat(3.2);
            } else if Some(piece.color) != model.game.winner {
                scale = Vec3::new(1.0, 0.45, 1.0);
            }
        }
        match piece.kind {
            PieceKind::Pawn => {
                add_part(
                    scene,
                    "pawn-body",
                    MESH_CYLINDER,
                    mat,
                    pos + Vec3::new(0.0, 0.35, 0.0),
                    rot,
                    Vec3::new(0.7, 1.0, 0.7).mul_elements(scale),
                );
                add_part(
                    scene,
                    "pawn-head",
                    MESH_SPHERE,
                    mat,
                    pos + Vec3::new(0.0, 0.85, 0.0),
                    rot,
                    Vec3::new(0.38, 0.38, 0.38).mul_elements(scale),
                );
                add_part(
                    scene,
                    "pawn-helm",
                    MESH_CONE,
                    MAT_STAFF,
                    pos + Vec3::new(0.0, 1.12, 0.0),
                    rot,
                    Vec3::new(0.28, 0.32, 0.28).mul_elements(scale),
                );
            }
            PieceKind::Rook => {
                add_part(
                    scene,
                    "rook-body",
                    MESH_BOX,
                    mat,
                    pos + Vec3::new(0.0, 0.55, 0.0),
                    rot,
                    Vec3::new(0.62, 0.9, 0.46).mul_elements(scale),
                );
                add_part(
                    scene,
                    "rook-head",
                    MESH_BOX,
                    mat,
                    pos + Vec3::new(0.0, 1.14, 0.0),
                    rot,
                    Vec3::new(0.5, 0.25, 0.5).mul_elements(scale),
                );
                add_part(
                    scene,
                    "rook-shoulder",
                    MESH_BOX,
                    MAT_STAFF,
                    pos + Vec3::new(-0.38, 0.78, 0.0),
                    rot,
                    Vec3::new(0.22, 0.2, 0.34).mul_elements(scale),
                );
                add_part(
                    scene,
                    "rook-shoulder",
                    MESH_BOX,
                    MAT_STAFF,
                    pos + Vec3::new(0.38, 0.78, 0.0),
                    rot,
                    Vec3::new(0.22, 0.2, 0.34).mul_elements(scale),
                );
            }
            PieceKind::Knight => {
                add_part(
                    scene,
                    "knight-body",
                    MESH_BOX,
                    mat,
                    pos + Vec3::new(0.0, 0.48, -0.08),
                    rot * Quat::from_euler_xyz(0.25, 0.0, 0.0),
                    Vec3::new(0.42, 0.78, 0.48).mul_elements(scale),
                );
                add_part(
                    scene,
                    "knight-neck",
                    MESH_BOX,
                    mat,
                    pos + Vec3::new(0.0, 0.88, 0.2),
                    rot * Quat::from_euler_xyz(-0.45, 0.0, 0.0),
                    Vec3::new(0.25, 0.55, 0.3).mul_elements(scale),
                );
                add_part(
                    scene,
                    "knight-rider",
                    MESH_SPHERE,
                    MAT_STAFF,
                    pos + Vec3::new(0.0, 0.98, -0.14),
                    rot,
                    splat(0.22).mul_elements(scale),
                );
            }
            PieceKind::Bishop => {
                add_part(
                    scene,
                    "bishop-robe",
                    MESH_CONE,
                    mat,
                    pos + Vec3::new(0.0, 0.58, 0.0),
                    rot,
                    Vec3::new(0.58, 1.3, 0.58).mul_elements(scale),
                );
                add_part(
                    scene,
                    "bishop-head",
                    MESH_SPHERE,
                    mat,
                    pos + Vec3::new(0.0, 1.22, 0.0),
                    rot,
                    splat(0.25).mul_elements(scale),
                );
                add_part(
                    scene,
                    "bishop-staff",
                    MESH_THIN_CYLINDER,
                    MAT_STAFF,
                    pos + Vec3::new(0.32, 0.82, 0.12),
                    rot * Quat::from_axis_angle(Vec3::Z, 0.2),
                    Vec3::new(1.0, 1.25, 1.0).mul_elements(scale),
                );
            }
            PieceKind::Queen => {
                add_part(
                    scene,
                    "queen-body",
                    MESH_CYLINDER,
                    mat,
                    pos + Vec3::new(0.0, 0.64, 0.0),
                    rot,
                    Vec3::new(0.62, 1.25, 0.62).mul_elements(scale),
                );
                add_part(
                    scene,
                    "queen-head",
                    MESH_SPHERE,
                    mat,
                    pos + Vec3::new(0.0, 1.32, 0.0),
                    rot,
                    splat(0.27).mul_elements(scale),
                );
                add_part(
                    scene,
                    "queen-halo",
                    MESH_TORUS,
                    MAT_CROWN,
                    pos + Vec3::new(0.0, 1.62, 0.0),
                    rot,
                    splat(0.56).mul_elements(scale),
                );
            }
            PieceKind::King => {
                add_part(
                    scene,
                    "king-body",
                    MESH_BOX,
                    mat,
                    pos + Vec3::new(0.0, 0.78, 0.0),
                    rot,
                    Vec3::new(0.62, 1.45, 0.46).mul_elements(scale),
                );
                add_part(
                    scene,
                    "king-cape",
                    MESH_BOX,
                    MAT_CAPE,
                    pos + Vec3::new(0.0, 0.74, -0.32),
                    rot,
                    Vec3::new(0.62, 1.25, 0.08).mul_elements(scale),
                );
                add_part(
                    scene,
                    "king-crown",
                    MESH_CYLINDER,
                    MAT_CROWN,
                    pos + Vec3::new(0.0, 1.58, 0.0),
                    rot,
                    Vec3::new(0.46, 0.42, 0.46).mul_elements(scale),
                );
            }
        }

        if model.settings.enable_magic_skills
            && matches!(
                piece.kind,
                PieceKind::Bishop | PieceKind::Knight | PieceKind::Queen
            )
        {
            add_part(
                scene,
                "magic-glow",
                MESH_TORUS,
                MAT_PORTAL,
                pos + Vec3::new(0.0, 0.18, 0.0),
                rot,
                splat(0.9),
            );
        }
    }

    fn add_part(
        scene: &mut SceneGraph,
        name: &str,
        mesh: MeshId,
        material: MaterialId,
        translation: Vec3,
        rotation: Quat,
        scale: Vec3,
    ) {
        scene.add(
            SceneNode::mesh(name, mesh, material)
                .transform(Transform::new(translation, rotation, scale))
                .layer(OBJECT_LAYER),
        );
    }

    fn add_particles(scene: &mut SceneGraph, runtime: &RoyalRenderer) {
        for particle in &runtime.particles {
            let life = 1.0 - particle.age / particle.ttl;
            scene.add(
                SceneNode::mesh("vfx", MESH_PARTICLE, particle.material)
                    .transform(Transform::new(
                        particle.origin,
                        Quat::IDENTITY,
                        splat(0.08 + 0.18 * life),
                    ))
                    .layer(OBJECT_LAYER),
            );
        }
    }

    fn register_assets(renderer: &mut Renderer) -> Result<BTreeMap<MeshId, Geometry>, JsValue> {
        let mut geometries = BTreeMap::new();
        register_mesh(
            renderer,
            &mut geometries,
            MESH_TILE,
            box_geometry(TILE_SIZE, 0.5, TILE_SIZE, 1, 1, 1),
        )?;
        register_mesh(
            renderer,
            &mut geometries,
            MESH_BASE,
            box_geometry(
                BOARD_SIZE as f32 * TILE_SIZE + 1.0,
                0.5,
                BOARD_SIZE as f32 * TILE_SIZE + 1.0,
                1,
                1,
                1,
            ),
        )?;
        register_mesh(
            renderer,
            &mut geometries,
            MESH_MARKER,
            cylinder_geometry(0.22, 0.22, 0.04, 28, 1, false),
        )?;
        register_mesh(
            renderer,
            &mut geometries,
            MESH_BOX,
            box_geometry(1.0, 1.0, 1.0, 1, 1, 1),
        )?;
        register_mesh(
            renderer,
            &mut geometries,
            MESH_SPHERE,
            sphere_geometry(1.0, 24, 12),
        )?;
        register_mesh(
            renderer,
            &mut geometries,
            MESH_CONE,
            cone_geometry(1.0, 1.0, 18, 1),
        )?;
        register_mesh(
            renderer,
            &mut geometries,
            MESH_CYLINDER,
            cylinder_geometry(1.0, 1.0, 1.0, 24, 1, false),
        )?;
        register_mesh(
            renderer,
            &mut geometries,
            MESH_THIN_CYLINDER,
            cylinder_geometry(0.04, 0.04, 1.0, 12, 1, false),
        )?;
        register_mesh(
            renderer,
            &mut geometries,
            MESH_TORUS,
            torus_geometry(0.65, 0.08, 32, 8),
        )?;
        register_mesh(
            renderer,
            &mut geometries,
            MESH_PARTICLE,
            box_geometry(1.0, 1.0, 1.0, 1, 1, 1),
        )?;

        register_material(renderer, MAT_BASE, 0x334155, 0.1, 0.5, 0x000000)?;
        register_material(renderer, MAT_SELECTED, 0x22c55e, 0.1, 0.25, 0x14532d)?;
        register_material(renderer, MAT_VALID, 0xfacc15, 0.1, 0.25, 0x713f12)?;
        register_material(renderer, MAT_CHECK, 0xef4444, 0.1, 0.25, 0x7f1d1d)?;
        register_material(renderer, MAT_LAST, 0x38bdf8, 0.05, 0.35, 0x075985)?;
        register_material(renderer, MAT_MARKER, 0x020617, 0.0, 0.6, 0x000000)?;
        register_material(renderer, MAT_PORTAL, 0xa855f7, 0.4, 0.18, 0x7e22ce)?;
        register_material(renderer, MAT_CAPE, 0xef4444, 0.0, 0.8, 0x7f1d1d)?;
        register_material(renderer, MAT_CROWN, 0xfbbf24, 1.0, 0.18, 0xb45309)?;
        register_material(renderer, MAT_STAFF, 0x64748b, 0.5, 0.3, 0x111827)?;
        register_material(renderer, MAT_RED_PARTICLE, 0xef4444, 0.0, 0.1, 0xef4444)?;
        register_material(renderer, MAT_WHITE_PARTICLE, 0xffffff, 0.0, 0.1, 0xffffff)?;
        register_material(renderer, MAT_GOLD_PARTICLE, 0xfbbf24, 0.0, 0.1, 0xfbbf24)?;

        for theme in Theme::ALL {
            let config = theme_config(theme);
            register_material(
                renderer,
                theme_tile_material(theme, false),
                config.light,
                0.1,
                0.32,
                0x000000,
            )?;
            register_material(
                renderer,
                theme_tile_material(theme, true),
                config.dark,
                0.1,
                0.32,
                0x000000,
            )?;
            register_material(
                renderer,
                theme_bg_material(theme),
                config.bg,
                0.0,
                0.95,
                config.accent,
            )?;
        }
        for skin in SkinType::ALL {
            let config = skin_config(skin);
            register_material(
                renderer,
                skin_material(skin, PlayerColor::White),
                config.white,
                config.metalness,
                config.roughness,
                config.emissive_white,
            )?;
            register_material(
                renderer,
                skin_material(skin, PlayerColor::Black),
                config.black,
                config.metalness,
                config.roughness,
                config.emissive_black,
            )?;
        }
        for weather in crate::types::Weather::ALL {
            register_material(
                renderer,
                weather_material(weather),
                weather_config(weather).color,
                0.0,
                0.2,
                weather_config(weather).color,
            )?;
        }
        renderer
            .register_ambient_light(LightId::new(1), AmbientLight::new(Color::WHITE, 0.35))
            .map_err(js_error)?;
        renderer
            .register_directional_light(
                LightId::new(2),
                DirectionalLight::new(Vec3::new(-0.6, -1.0, -0.35), Color::WHITE, 3.0),
            )
            .map_err(js_error)?;
        Ok(geometries)
    }

    fn register_mesh(
        renderer: &mut Renderer,
        geometries: &mut BTreeMap<MeshId, Geometry>,
        id: MeshId,
        geometry: Geometry,
    ) -> Result<(), JsValue> {
        renderer.register_mesh(id, &geometry).map_err(js_error)?;
        geometries.insert(id, geometry);
        Ok(())
    }

    fn register_material(
        renderer: &mut Renderer,
        id: MaterialId,
        color: u32,
        metalness: f32,
        roughness: f32,
        emissive: u32,
    ) -> Result<(), JsValue> {
        let mut material = PbrMaterial::new()
            .albedo(Color::from_hex(color).to_linear())
            .metallic_roughness(metalness, roughness);
        let e = Color::from_hex(emissive).to_linear();
        material.emissive = Vec3::new(e.r, e.g, e.b);
        renderer
            .register_pbr_material(id, &material)
            .map_err(js_error)
    }

    fn theme_tile_material(theme: Theme, dark: bool) -> MaterialId {
        MaterialId::new(100 + theme_index(theme) * 3 + if dark { 1 } else { 0 })
    }

    fn theme_bg_material(theme: Theme) -> MaterialId {
        MaterialId::new(100 + theme_index(theme) * 3 + 2)
    }

    fn skin_material(skin: SkinType, color: PlayerColor) -> MaterialId {
        MaterialId::new(
            200 + skin_index(skin) * 2 + if color == PlayerColor::Black { 1 } else { 0 },
        )
    }

    fn weather_material(weather: crate::types::Weather) -> MaterialId {
        MaterialId::new(300 + weather as u64)
    }

    fn theme_index(theme: Theme) -> u64 {
        match theme {
            Theme::Classic => 0,
            Theme::Volcano => 1,
            Theme::Ice => 2,
            Theme::Cyberpunk => 3,
            Theme::Forest => 4,
            Theme::Castle => 5,
            Theme::Desert => 6,
            Theme::Galaxy => 7,
        }
    }

    fn skin_index(skin: SkinType) -> u64 {
        match skin {
            SkinType::Classic => 0,
            SkinType::Marble => 1,
            SkinType::Gold => 2,
            SkinType::Crystal => 3,
            SkinType::Bronze => 4,
            SkinType::Shadow => 5,
            SkinType::Neon => 6,
            SkinType::Wood => 7,
        }
    }

    fn desired_camera(model: &AppModel) -> Vec3 {
        if model.screen == crate::types::AppScreen::Menu {
            let t = js_sys::Date::now() as f32 * 0.0002;
            return Vec3::new(t.sin() * 22.0, 14.0, t.cos() * 22.0);
        }
        if model.game.cinematic_mode == CinematicMode::GiantKingEnding {
            return Vec3::new(
                0.0,
                2.5,
                if model.game.winner == Some(PlayerColor::White) {
                    20.0
                } else {
                    -20.0
                },
            );
        }
        if model.game.cinematic_mode == CinematicMode::KillCam && model.settings.cinematic_camera {
            if let Some(last) = &model.game.last_move {
                let pos = vec3_from_square(&last.to);
                return pos + Vec3::new(4.0, 4.5, 4.0);
            }
        }
        match model.game.camera_mode {
            CameraMode::Top => Vec3::new(0.0, 18.0, 0.001),
            CameraMode::Cinematic => Vec3::new(-8.0, 8.0, 8.0),
            CameraMode::Orbit => {
                if matches!(
                    model.game.game_mode,
                    GameMode::PlayerVsAi | GameMode::Story | GameMode::Boss
                ) {
                    Vec3::new(0.0, 12.0, 12.0)
                } else if model.game.turn == PlayerColor::White {
                    Vec3::new(0.0, 12.0, 12.0)
                } else {
                    Vec3::new(0.0, 12.0, -12.0)
                }
            }
        }
    }

    fn vec3_from_square(square: &str) -> Vec3 {
        let (x, y, z) = board_position(square);
        Vec3::new(x, y + 0.5, z)
    }

    fn splat(value: f32) -> Vec3 {
        Vec3::new(value, value, value)
    }

    fn canvas_size(canvas: &web_sys::HtmlCanvasElement) -> (u32, u32) {
        let width = canvas.client_width().max(1) as u32;
        let height = canvas.client_height().max(1) as u32;
        (width, height)
    }

    fn js_error(error: impl std::fmt::Display) -> JsValue {
        JsValue::from_str(&error.to_string())
    }

    pub(super) fn js_value_text(value: &JsValue) -> String {
        value.as_string().unwrap_or_else(|| format!("{value:?}"))
    }

    pub(super) fn start_loop(runtime: Rc<RefCell<Option<RenderRuntime>>>) {
        fn request(runtime: Rc<RefCell<Option<RenderRuntime>>>) {
            let cb = Closure::once_into_js(move |timestamp: f64| {
                if let Some(renderer) = runtime.borrow_mut().as_mut() {
                    if let Err(error) = renderer.tick(timestamp) {
                        renderer.state().update(|model| {
                            model.renderer_error = Some(format!("{error:?}"));
                        });
                    }
                }
                request(runtime);
            });
            let _ = web_sys::window()
                .unwrap()
                .request_animation_frame(cb.as_ref().unchecked_ref());
        }
        request(runtime);
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_impl::{
    CanvasFallback, RenderRuntime, RoyalRenderer, WebGlRoyalRenderer, js_value_text, start_loop,
};
