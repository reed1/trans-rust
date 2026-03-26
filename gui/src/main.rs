use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{ActiveTheme, Root, Theme};

use trans_core::search::{SearchIndex, SearchOutput};

actions!(trans_gui, [ClearInput]);

const SCROLL_AMOUNT: f32 = 400.0;

static VERBOSE: AtomicBool = AtomicBool::new(false);

macro_rules! log {
    ($($arg:tt)*) => {
        if VERBOSE.load(Ordering::Relaxed) {
            eprintln!($($arg)*);
        }
    };
}

struct AppView {
    input: Entity<InputState>,
    output: Option<SearchOutput>,
    search_index: SearchIndex,
    scroll_handle: ScrollHandle,
    debounce_task: Option<Task<()>>,
    _subscription: Subscription,
}

impl AppView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search..."));
        input.update(cx, |input, cx| input.focus(window, cx));

        let subscription = cx.subscribe(&input, |this: &mut AppView, input_entity, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                let text = input_entity.read(cx).value().to_string();
                if text.len() >= 3 {
                    this.debounce_task = Some(cx.spawn(async move |weak, cx| {
                        Timer::after(Duration::from_millis(300)).await;
                        weak.update(cx, |this, cx| {
                            this.output = Some(this.search_index.search(&text, None));
                            this.scroll_handle.set_offset(point(px(0.0), px(0.0)));
                            cx.notify();
                        })
                        .ok();
                    }));
                } else {
                    this.debounce_task = None;
                    this.output = None;
                    cx.notify();
                }
            }
        });

        AppView {
            input,
            output: None,
            search_index: SearchIndex::open(),
            scroll_handle: ScrollHandle::new(),
            debounce_task: None,
            _subscription: subscription,
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

        let mut content = div().flex().flex_col().gap(px(2.0)).px(px(12.0)).pt(px(4.0));

        if let Some(output) = &self.output {
            if output.entries.is_empty() {
                content = content.child(
                    div()
                        .text_size(px(14.0))
                        .text_color(theme.muted_foreground)
                        .child(format!("No results for \"{}\"", output.query)),
                );
            } else if !output.exact {
                content = content.child(
                    div()
                        .text_size(px(14.0))
                        .text_color(theme.link)
                        .pb(px(4.0))
                        .child(format!("Did you mean: {}?", output.entries[0].word)),
                );
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

            content = content.child(
                div()
                    .mt(px(12.0))
                    .h(px(1.0))
                    .bg(theme.muted_foreground),
            );
        }

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .on_action(cx.listener(Self::clear_input))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                log!("key_down: key={:?} ctrl={}", event.keystroke.key, event.keystroke.modifiers.control);
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
            .child(
                div()
                    .p(px(12.0))
                    .child(Input::new(&self.input)),
            )
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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-v" || a == "--verbose") {
        VERBOSE.store(true, Ordering::Relaxed);
    }

    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);

        cx.bind_keys([
            KeyBinding::new(
                "ctrl-w",
                gpui_component::input::DeleteToPreviousWordStart,
                Some("Input"),
            ),
            KeyBinding::new("ctrl-l", ClearInput, None),
        ]);

        let options = WindowOptions {
            window_min_size: Some(size(px(400.0), px(300.0))),
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
