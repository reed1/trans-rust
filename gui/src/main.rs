use std::time::Duration;

use gpui::*;
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{ActiveTheme, Root, Theme};

use trans_core::search::{SearchIndex, SearchOutput};

struct AppView {
    input: Entity<InputState>,
    output: Option<SearchOutput>,
    search_index: SearchIndex,
    debounce_task: Option<Task<()>>,
    _subscription: Subscription,
}

impl AppView {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(window, cx).placeholder("Search..."));

        let subscription = cx.subscribe(&input, |this: &mut AppView, input_entity, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                let text = input_entity.read(cx).value().to_string();
                if text.len() >= 3 {
                    this.debounce_task = Some(cx.spawn(async move |weak, cx| {
                        Timer::after(Duration::from_millis(300)).await;
                        weak.update(cx, |this, cx| {
                            this.output = Some(this.search_index.search(&text, None));
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
            debounce_task: None,
            _subscription: subscription,
        }
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let mut content = div().flex().flex_col().gap(px(4.0)).p(px(12.0));

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
                        .text_color(theme.warning)
                        .pb(px(8.0))
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
                        .pl(px(16.0))
                        .text_size(px(14.0))
                        .text_color(theme.foreground)
                        .child(format!("{}", r.definition))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(theme.muted_foreground)
                                .child(format!("[{}]", r.source)),
                        ),
                );
            }
        }

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .child(
                div()
                    .p(px(12.0))
                    .child(Input::new(&self.input)),
            )
            .child(
                div()
                    .flex_1()
                    .overflow_y_scrollbar()
                    .child(content),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);

        let options = WindowOptions {
            window_min_size: Some(size(px(400.0), px(300.0))),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
            let view = cx.new(|cx| AppView::new(window, cx));
            cx.new(|cx| Root::new(view, window, cx))
        })
        .unwrap();
    });
}
