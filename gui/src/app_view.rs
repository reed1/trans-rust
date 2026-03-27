use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{ActiveTheme, Root, Theme};

use trans_core::provider::{GoogleProvider, LocalProvider, Provider};
use trans_core::search::SearchOutput;

actions!(trans_gui, [ClearInput, PlayAudio, ApplySuggestion]);

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 550.0;
const SCROLL_AMOUNT: f32 = 400.0;
const LOCAL_MIN_LENGTH: usize = 3;

struct AppView {
    input: Entity<InputState>,
    output: Option<SearchOutput>,
    provider: Arc<LocalProvider>,
    scroll_handle: ScrollHandle,
    debounce_task: Option<Task<()>>,
    google_task: Option<Task<()>>,
    audio_task: Option<Task<()>>,
    audio_cache: Arc<std::sync::Mutex<HashMap<(String, String), Vec<u8>>>>,
    _subscription: Subscription,
}

pub fn run() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);

        cx.bind_keys([
            KeyBinding::new(
                "ctrl-w",
                gpui_component::input::DeleteToPreviousWordStart,
                Some("Input"),
            ),
            KeyBinding::new(
                "ctrl-h",
                gpui_component::input::Backspace,
                Some("Input"),
            ),
            KeyBinding::new("ctrl-l", ClearInput, None),
            KeyBinding::new("ctrl-o", PlayAudio, None),
            KeyBinding::new("ctrl-m", ApplySuggestion, None),
        ]);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)),
                cx,
            ))),
            app_id: Some("trans-gui".into()),
            kind: WindowKind::PopUp,
            focus: true,
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            Theme::change(gpui_component::theme::ThemeMode::Dark, Some(window), cx);
            let view = cx.new(|cx| AppView::new(window, cx));
            cx.new(|cx| Root::new(view, window, cx))
        })
        .unwrap();
    });
}

impl AppView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search..."));
        input.update(cx, |input, cx| input.focus(window, cx));

        let subscription = cx.subscribe(&input, Self::on_input_event);

        AppView {
            input,
            output: None,
            provider: Arc::new(LocalProvider::new()),
            scroll_handle: ScrollHandle::new(),
            debounce_task: None,
            google_task: None,
            audio_task: None,
            audio_cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
            _subscription: subscription,
        }
    }

    fn on_input_event(
        &mut self,
        input_entity: Entity<InputState>,
        event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => self.on_change(&input_entity, cx),
            InputEvent::PressEnter { .. } => self.on_enter(&input_entity, cx),
            _ => {}
        }
    }

    fn on_change(&mut self, input_entity: &Entity<InputState>, cx: &mut Context<Self>) {
        let text = input_entity.read(cx).value().to_string();
        self.google_task = None;
        if text.len() >= LOCAL_MIN_LENGTH {
            let provider = Arc::clone(&self.provider);
            self.debounce_task = Some(cx.spawn(async move |weak, cx| {
                Timer::after(Duration::from_millis(300)).await;
                let output = smol::unblock(move || {
                    log!("local search: {:?}", text);
                    provider.search(&text, ("", ""))
                })
                .await;
                weak.update(cx, |this, cx| {
                    this.output = Some(output);
                    this.scroll_handle.set_offset(point(px(0.0), px(0.0)));
                    cx.notify();
                })
                .ok();
            }));
        } else {
            self.debounce_task = None;
            self.output = None;
            cx.notify();
        }
    }

    fn on_enter(&mut self, input_entity: &Entity<InputState>, cx: &mut Context<Self>) {
        let query = input_entity.read(cx).value().to_string();
        if query.is_empty() {
            return;
        }
        self.google_task = Some(cx.spawn(async move |weak, cx| {
            let query_check = query.clone();
            let output = smol::unblock(move || {
                log!("google search: {:?}", query);
                let q1 = query.clone();
                let q2 = query.clone();
                let (id_en, en_id) = std::thread::scope(|s| {
                    let h1 = s.spawn(move || GoogleProvider.search(&q1, ("id", "en")));
                    let h2 = s.spawn(move || GoogleProvider.search(&q2, ("en", "id")));
                    (h1.join().ok(), h2.join().ok())
                });
                let empty = || SearchOutput {
                    query: query.clone(),
                    exact: true,
                    suggestion: None,
                    entries: vec![],
                };
                let id_en = id_en.unwrap_or_else(&empty);
                let en_id = en_id.unwrap_or_else(empty);
                let mut entries = id_en.entries;
                entries.extend(en_id.entries);
                SearchOutput {
                    query,
                    exact: id_en.exact && en_id.exact,
                    suggestion: id_en.suggestion.or(en_id.suggestion),
                    entries,
                }
            })
            .await;
            weak.update(cx, |this, cx| {
                let current = this.input.read(cx).value().to_string();
                if current == query_check {
                    this.output = Some(output);
                    this.scroll_handle.set_offset(point(px(0.0), px(0.0)));
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    fn play_audio(&mut self, _: &PlayAudio, _window: &mut Window, cx: &mut Context<Self>) {
        let query = self.input.read(cx).value().to_string();
        if query.is_empty() {
            return;
        }

        let lang = self
            .output
            .as_ref()
            .and_then(|o| o.entries.first())
            .map(|e| e.source_lang.clone())
            .unwrap_or_else(|| "en".to_string());

        let cache = Arc::clone(&self.audio_cache);
        let key = (query.clone(), lang.clone());

        self.audio_task = Some(cx.spawn(async move |_weak, _cx| {
            let mp3 = {
                let cached = cache.lock().unwrap().get(&key).cloned();
                if let Some(data) = cached {
                    data
                } else {
                    let data = smol::unblock({
                        let key = key.clone();
                        move || fetch_tts(&key.0, &key.1)
                    })
                    .await;
                    cache.lock().unwrap().insert(key, data.clone());
                    data
                }
            };

            smol::unblock(move || {
                let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
                let source = rodio::Decoder::new(Cursor::new(mp3)).unwrap();
                let sink = rodio::Sink::try_new(&handle).unwrap();
                sink.append(source);
                sink.sleep_until_end();
            })
            .await;
        }));
    }

    fn apply_suggestion(
        &mut self,
        _: &ApplySuggestion,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let suggestion = self
            .output
            .as_ref()
            .and_then(|o| o.suggestion.clone())
            .unwrap_or_default();
        if !suggestion.is_empty() {
            self.input
                .update(cx, |input, cx| input.set_value(&suggestion, window, cx));
        }
    }

    fn clear_input(&mut self, _: &ClearInput, window: &mut Window, cx: &mut Context<Self>) {
        self.input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.output = None;
        self.debounce_task = None;
        cx.notify();
    }

    fn scroll_by(&mut self, delta: f32, cx: &mut Context<Self>) {
        let current = self.scroll_handle.offset();
        self.scroll_handle
            .set_offset(point(current.x, current.y + px(delta)));
        cx.notify();
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let mut content = div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .px(px(12.0))
            .pt(px(4.0));

        if let Some(output) = &self.output {
            content = self.render_output(content, output, &theme);
        }

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .on_action(cx.listener(Self::clear_input))
            .on_action(cx.listener(Self::play_audio))
            .on_action(cx.listener(Self::apply_suggestion))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                log!(
                    "key_down: key={:?} ctrl={}",
                    event.keystroke.key,
                    event.keystroke.modifiers.control
                );
                if event.keystroke.modifiers.control {
                    match event.keystroke.key.as_str() {
                        "u" => {
                            log!("scroll_up: offset={:?}", this.scroll_handle.offset());
                            this.scroll_by(SCROLL_AMOUNT, cx);
                        }
                        "d" => {
                            log!("scroll_down: offset={:?}", this.scroll_handle.offset());
                            this.scroll_by(-SCROLL_AMOUNT, cx);
                        }
                        _ => {}
                    }
                }
            }))
            .child(div().p(px(12.0)).child(Input::new(&self.input)))
            .child(
                div()
                    .id("results")
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .child(content),
            )
    }
}

impl AppView {
    fn render_output(
        &self,
        mut content: Div,
        output: &SearchOutput,
        theme: &gpui_component::theme::Theme,
    ) -> Div {
        if output.entries.is_empty() {
            return content.child(
                div()
                    .text_size(px(14.0))
                    .text_color(theme.muted_foreground)
                    .child(format!("No results for \"{}\"", output.query)),
            );
        }

        if !output.exact {
            if let Some(suggestion) = &output.suggestion {
                content = content.child(
                    div()
                        .text_size(px(14.0))
                        .text_color(theme.link)
                        .pb(px(4.0))
                        .child(format!("Did you mean: {}?", suggestion)),
                );
            }
        } else {
            content = content.child(
                div()
                    .text_size(px(13.0))
                    .text_color(theme.muted_foreground)
                    .pb(px(4.0))
                    .child(format!(
                        "{} result{}",
                        output.entries.len(),
                        if output.entries.len() == 1 { "" } else { "s" }
                    )),
            );
        }

        let mut last_word = String::new();
        for r in &output.entries {
            if r.word != last_word {
                let mut header = div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(6.0))
                    .pt(px(6.0));

                header = header.child(
                    div()
                        .text_size(px(16.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.foreground)
                        .child(r.word.clone()),
                );

                if !r.pronunciation.is_empty() {
                    header = header.child(
                        div()
                            .text_size(px(14.0))
                            .text_color(theme.muted_foreground)
                            .child(r.pronunciation.clone()),
                    );
                }

                if !r.pos.is_empty() {
                    header = header.child(
                        div()
                            .text_size(px(14.0))
                            .text_color(theme.muted_foreground)
                            .child(format!("({})", r.pos)),
                    );
                }

                content = content.child(header);
                last_word = r.word.clone();
            }

            content = content.child(
                div()
                    .flex()
                    .flex_row()
                    .items_baseline()
                    .gap(px(8.0))
                    .pl(px(16.0))
                    .child(
                        div()
                            .text_size(px(14.0))
                            .text_color(theme.foreground)
                            .child(r.definition.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.muted_foreground)
                            .child(format!("[{}]", r.source)),
                    ),
            );
        }

        content.child(
            div()
                .mt(px(12.0))
                .h(px(1.0))
                .bg(theme.muted_foreground),
        )
    }
}

fn fetch_tts(text: &str, lang: &str) -> Vec<u8> {
    log!("tts fetch: text={:?} lang={:?}", text, lang);
    ureq::get("https://translate.google.com/translate_tts")
        .query("ie", "UTF-8")
        .query("client", "tw-ob")
        .query("tl", lang)
        .query("q", text)
        .call()
        .unwrap()
        .body_mut()
        .read_to_vec()
        .unwrap()
}
