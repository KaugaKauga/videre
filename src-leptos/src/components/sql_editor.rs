use leptos::{
    ev, html,
    leptos_dom::helpers::{request_animation_frame, window_event_listener},
    portal::Portal,
    prelude::*,
};
use wasm_bindgen::{closure::Closure, JsCast};

use super::{
    sql_completion::{
        byte_to_utf16, completion_context, required_relations, suggestions, utf16_to_byte,
        CompletionCandidate, CompletionContext,
    },
    sql_highlight::keyword_ranges,
};
use crate::stores::db_store::DbStore;

const MAX_HIGHLIGHT_BYTES: usize = 128 * 1024;

#[derive(Clone, PartialEq)]
struct CompletionMenu {
    sql: String,
    caret: u32,
    context: CompletionContext,
    candidates: Vec<CompletionCandidate>,
}

fn replacement_sql(sql: &str, context: &CompletionContext, insertion: &str) -> String {
    let mut edited = sql.to_owned();
    edited.replace_range(context.replacement.clone(), insertion);
    edited
}

fn insert_completion(input: &web_sys::HtmlTextAreaElement, menu: &CompletionMenu, insertion: &str) {
    let start = byte_to_utf16(&menu.sql, menu.context.replacement.start);
    let end = byte_to_utf16(&menu.sql, menu.context.replacement.end);
    let expected = replacement_sql(&menu.sql, &menu.context, insertion);
    let _ = input.set_selection_range(start, end);
    // insertText participates in native undo on supported webviews. Range-text is
    // the fallback, not a value write-back on every ordinary input event.
    if let Some(document) = input.owner_document() {
        let _ = document
            .unchecked_ref::<web_sys::HtmlDocument>()
            .exec_command_with_show_ui_and_value("insertText", false, insertion);
    }
    if input.value() != expected {
        let _ = input.set_range_text_with_start_and_end(insertion, start, end);
    }
    let caret = start + insertion.encode_utf16().count() as u32;
    let _ = input.set_selection_range(caret, caret);
}

fn highlighted_parts(sql: &str) -> Vec<(String, bool)> {
    let mut parts = Vec::new();
    let mut end = 0;
    for range in keyword_ranges(sql) {
        if end < range.start {
            parts.push((sql[end..range.start].to_owned(), false));
        }
        parts.push((sql[range.clone()].to_owned(), true));
        end = range.end;
    }
    if end < sql.len() {
        parts.push((sql[end..].to_owned(), false));
    }
    parts
}

type HighlightLine = Vec<(String, bool)>;

/// Splits highlighted parts into lines, each keeping its trailing newline, and
/// numbers repeated lines so every line has a unique key. Rendering keyed lines
/// rebuilds only edited lines instead of the whole layer on every frame.
fn highlighted_lines(sql: &str) -> Vec<(HighlightLine, usize)> {
    let mut lines = Vec::new();
    let mut line = HighlightLine::new();
    for (text, keyword) in highlighted_parts(sql) {
        // Keywords never contain a newline.
        for piece in text.split_inclusive('\n') {
            line.push((piece.to_owned(), keyword));
            if piece.ends_with('\n') {
                lines.push(std::mem::take(&mut line));
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    let mut seen = std::collections::HashMap::<HighlightLine, usize>::new();
    lines
        .into_iter()
        .map(|line| {
            let occurrence = seen.entry(line.clone()).or_default();
            *occurrence += 1;
            (line, *occurrence)
        })
        .collect()
}

/// Native editing with a decorative highlight layer; never rewrites the input
/// after a keystroke, preserving the browser's caret, selection and undo history.
#[component]
pub fn SqlEditor(id: String, sql: RwSignal<String>, disabled: Signal<bool>) -> impl IntoView {
    let db = expect_context::<DbStore>();
    let menu = RwSignal::new(None::<CompletionMenu>);
    let selected = RwSignal::new(0usize);
    let dismissed = RwSignal::new(false);
    let position = RwSignal::new(String::new());
    let list_id = format!("{id}-completions");
    let option_id = StoredValue::new(list_id.clone());
    let popup_id = StoredValue::new(list_id.clone());
    let active_option_id = list_id.clone();
    let controls_id = list_id.clone();
    let initial_sql = sql.get_untracked();
    let displayed_sql = RwSignal::new(initial_sql.clone());
    let composing = RwSignal::new(false);
    let frame_pending = RwSignal::new(false);
    let input_ref = NodeRef::<html::Textarea>::new();
    let highlight_ref = NodeRef::<html::Pre>::new();
    let prefix_ref = NodeRef::<html::Span>::new();
    let marker_ref = NodeRef::<html::Span>::new();
    let list_ref = NodeRef::<html::Div>::new();
    // Very large scripts remain editable without constructing thousands of spans.
    let highlighted =
        Memo::new(move |_| displayed_sql.with(|text| text.len() <= MAX_HIGHLIGHT_BYTES));

    let position_menu = move || {
        let (Some(input), Some(prefix), Some(marker), Some(window)) = (
            input_ref.get_untracked(),
            prefix_ref.get_untracked(),
            marker_ref.get_untracked(),
            web_sys::window(),
        ) else {
            return;
        };
        let Some((caret, count)) = menu.with_untracked(|menu| {
            menu.as_ref()
                .map(|menu| (menu.caret, menu.candidates.len()))
        }) else {
            return;
        };
        let text = input.value();
        prefix.set_text_content(Some(&text[..utf16_to_byte(&text, caret)]));
        // A prefix-only mirror cannot scroll as far as the full textarea. Measure
        // it unscrolled, then subtract the native scroll offsets numerically.
        let marker_rect = marker.get_bounding_client_rect();
        let input_rect = input.get_bounding_client_rect();
        let x = marker_rect.left() - f64::from(input.scroll_left());
        let y = marker_rect.top() - f64::from(input.scroll_top());
        let line_height = marker_rect.height().max(20.0);
        if x < input_rect.left()
            || x > input_rect.right()
            || y < input_rect.top()
            || y + line_height > input_rect.bottom()
        {
            menu.set(None);
            return;
        }
        let viewport_width = window
            .inner_width()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(800.0);
        let viewport_height = window
            .inner_height()
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(600.0);
        let width = 380.0f64.min((viewport_width - 16.0).max(0.0));
        let left = x.min(viewport_width - width - 8.0).max(8.0);
        let below = (viewport_height - y - line_height - 12.0).max(0.0);
        let above = (y - 12.0).max(0.0);
        let desired = (count as f64 * 32.0 + 40.0).min(300.0);
        let (top, height) = if below >= desired || below >= above {
            (y + line_height + 4.0, desired.min(below))
        } else {
            let height = desired.min(above);
            (y - height - 4.0, height)
        };
        position.set(format!(
            "left:{left}px;top:{top}px;width:{width}px;max-height:{height}px"
        ));
    };
    let refresh_menu = move || {
        let Some(input) = input_ref.get_untracked() else {
            return;
        };
        if dismissed.get_untracked()
            || composing.get_untracked()
            || disabled.get_untracked()
            || !input.matches(":focus").unwrap_or(false)
            || !db.is_connected.get_untracked()
            || db.is_loading.get_untracked()
        {
            menu.set(None);
            return;
        }
        let text = input.value();
        let caret = input.selection_start().ok().flatten().unwrap_or(0);
        if text.len() > MAX_HIGHLIGHT_BYTES || input.selection_end().ok().flatten() != Some(caret) {
            menu.set(None);
            return;
        }
        let Some(context) = completion_context(&text, caret) else {
            menu.set(None);
            return;
        };
        let (keys, candidates) = db.tables.with_untracked(|tables| {
            let keys = required_relations(&context, tables);
            let candidates = db
                .relation_structures
                .with_untracked(|structures| suggestions(&context, tables, structures));
            (keys, candidates)
        });
        for key in keys {
            db.request_relation_structure(&key.schema, &key.relation);
        }
        if candidates.is_empty() {
            menu.set(None);
            return;
        }
        let next = CompletionMenu {
            sql: text,
            caret,
            context,
            candidates,
        };
        let unchanged = menu.with_untracked(|menu| menu.as_ref() == Some(&next));
        if !unchanged {
            selected.set(0);
            menu.set(Some(next));
        }
        position_menu();
    };
    let queue_highlight = move || {
        if frame_pending.get_untracked() || composing.get_untracked() {
            return;
        }
        frame_pending.set(true);
        request_animation_frame(move || {
            // A tab may have unmounted before its pending frame is delivered.
            if frame_pending.try_get_untracked() != Some(true) {
                return;
            }
            frame_pending.set(false);
            if !composing.get_untracked() {
                if let Some(input) = input_ref.get_untracked() {
                    let text = input.value();
                    if displayed_sql.with_untracked(|shown| shown != &text) {
                        displayed_sql.set(text);
                    }
                    refresh_menu();
                }
            }
        });
    };
    let sync_scroll = move || {
        if let (Some(input), Some(layer)) =
            (input_ref.get_untracked(), highlight_ref.get_untracked())
        {
            // Native scrollbars consume viewport space. Match the client area,
            // plus the shared 1px borders, so bottom/right offsets cannot clamp early.
            let style = layer.unchecked_ref::<web_sys::HtmlElement>().style();
            for (property, pixels) in [
                ("width", input.client_width() + 2),
                ("height", input.client_height() + 2),
            ] {
                let value = format!("{pixels}px");
                if style.get_property_value(property).ok().as_deref() != Some(&value) {
                    let _ = style.set_property(property, &value);
                }
            }
            layer.set_scroll_top(input.scroll_top());
            layer.set_scroll_left(input.scroll_left());
        }
        position_menu();
    };
    let accept = move |index: usize| {
        let (Some(input), Some(current)) = (input_ref.get_untracked(), menu.get_untracked()) else {
            return;
        };
        if composing.get_untracked()
            || disabled.get_untracked()
            || input.value() != current.sql
            || input.selection_start().ok().flatten() != Some(current.caret)
            || input.selection_end().ok().flatten() != Some(current.caret)
        {
            menu.set(None);
            return;
        }
        if let Some(candidate) = current.candidates.get(index) {
            insert_completion(&input, &current, &candidate.insertion);
            sql.set(input.value());
            dismissed.set(true);
            menu.set(None);
            queue_highlight();
        }
    };
    let on_keydown = move |event: web_sys::KeyboardEvent| {
        if event.is_composing()
            || composing.get_untracked()
            || event.ctrl_key()
            || event.meta_key()
            || event.alt_key()
            || event.shift_key()
        {
            return;
        }
        let Some(input) = input_ref.get_untracked() else {
            return;
        };
        let current_count = || {
            menu.with_untracked(|menu| {
                menu.as_ref()
                    .filter(|menu| {
                        input.value() == menu.sql
                            && input.selection_start().ok().flatten() == Some(menu.caret)
                            && input.selection_end().ok().flatten() == Some(menu.caret)
                    })
                    .map_or(0, |menu| menu.candidates.len())
            })
        };
        let mut count = current_count();
        if count == 0 && menu.with_untracked(Option::is_some) {
            // A visible menu refreshes once per frame. A key pressed sooner must
            // act on the current text and caret, not on the previous snapshot.
            refresh_menu();
            count = current_count();
        }
        if count == 0 {
            return;
        }
        match event.key().as_str() {
            "Tab" => accept(selected.get_untracked()),
            "ArrowDown" => selected.set((selected.get_untracked() + 1) % count),
            "ArrowUp" => selected.set((selected.get_untracked() + count - 1) % count),
            "Escape" => {
                dismissed.set(true);
                menu.set(None);
            }
            _ => return,
        }
        event.prevent_default();
        event.stop_propagation();
    };
    let resize_listener = window_event_listener(ev::resize, move |_| position_menu());
    on_cleanup(move || resize_listener.remove());
    // Catches every caret move, including those without a matching keyup such
    // as Cmd+Arrow on macOS. Captured on the document because webviews fire it
    // either there or, bubbling, at the textarea itself.
    let selection_callback = StoredValue::new_local(Closure::<dyn FnMut()>::new(move || {
        let Some(input) = input_ref.try_get_untracked().flatten() else {
            return;
        };
        let focused = input
            .owner_document()
            .and_then(|document| document.active_element())
            .is_some_and(|active| active.is_same_node(Some(&input)));
        if focused {
            queue_highlight();
        }
    }));
    let selection_document = web_sys::window().and_then(|window| window.document());
    if let Some(document) = &selection_document {
        selection_callback.with_value(|callback| {
            let _ = document.add_event_listener_with_callback_and_bool(
                "selectionchange",
                callback.as_ref().unchecked_ref(),
                true,
            );
        });
    }
    on_cleanup(move || {
        if let Some(document) = selection_document {
            selection_callback.with_value(|callback| {
                let _ = document.remove_event_listener_with_callback_and_bool(
                    "selectionchange",
                    callback.as_ref().unchecked_ref(),
                    true,
                );
            });
        }
    });
    // The textarea is uncontrolled while typing; programmatic draft changes
    // (anything other than this editor's own input) still have to reach it.
    Effect::new(move |_| {
        let text = sql.get();
        if let Some(input) = input_ref.get() {
            if input.value() != text {
                input.set_value(&text);
                dismissed.set(true);
                menu.set(None);
                queue_highlight();
            }
        }
    });
    Effect::new(move |_| {
        db.tables.track();
        db.relation_structures.track();
        db.is_connected.track();
        db.is_loading.track();
        disabled.track();
        queue_highlight();
    });
    Effect::new(move |_| {
        let index = selected.get();
        menu.track();
        if let Some(list) = list_ref.get() {
            if let Some(option) = list.children().item(index as u32) {
                let scroll = web_sys::ScrollIntoViewOptions::new();
                scroll.set_block(web_sys::ScrollLogicalPosition::Nearest);
                option.scroll_into_view_with_scroll_into_view_options(&scroll);
            }
        }
    });
    let resize_callback = StoredValue::new_local(Closure::<
        dyn FnMut(js_sys::Array, web_sys::ResizeObserver),
    >::new(move |_, _| {
        if frame_pending.try_get_untracked().is_some() {
            sync_scroll();
        }
    }));
    let resize_observer = StoredValue::new_local(resize_callback.with_value(|callback| {
        web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()).ok()
    }));
    on_cleanup(move || {
        resize_observer.with_value(|observer| {
            if let Some(observer) = observer {
                observer.disconnect();
            }
        });
    });
    Effect::new(move |_| {
        if let Some(input) = input_ref.get() {
            resize_observer.with_value(|observer| {
                if let Some(observer) = observer {
                    observer.observe(&input);
                }
            });
        }
    });
    Effect::new(move |_| {
        displayed_sql.track();
        input_ref.track();
        highlight_ref.track();
        sync_scroll();
    });

    view! {
        <div class="sql-editor"
            class:highlighted=move || highlighted.get()
            class:composing=move || composing.get()
        >
            <pre class="sql-highlight" aria-hidden="true" node_ref=highlight_ref>
                <For
                    each=move || {
                        if highlighted.get() {
                            displayed_sql.with(|text| highlighted_lines(text))
                        } else {
                            Vec::new()
                        }
                    }
                    key=|line| line.clone()
                    children=|(line, _)| {
                        line.into_iter().map(|(text, keyword)| {
                            if keyword {
                                view! { <span class="sql-keyword">{text}</span> }.into_any()
                            } else {
                                text.into_any()
                            }
                        }).collect_view()
                    }
                />
                // Keep the final empty line scrollable when the input ends in a newline.
                {move || highlighted.get().then_some("\n")}
            </pre>
            <pre class="sql-caret-mirror" aria-hidden="true"><span node_ref=prefix_ref></span><span node_ref=marker_ref>"\u{200b}"</span></pre>
            <textarea
                id=id
                class="query-sql-input"
                node_ref=input_ref
                rows="8"
                wrap="off"
                spellcheck="false"
                autocomplete="off"
                autocapitalize="off"
                aria-autocomplete="list"
                aria-haspopup="listbox"
                aria-expanded=move || menu.with(|menu| menu.is_some()).to_string()
                aria-controls=controls_id
                aria-activedescendant=move || menu.with(|menu| menu.as_ref().map(|_| format!("{active_option_id}-{}", selected.get())))
                disabled=move || disabled.get()
                prop:value=initial_sql
                on:input=move |event| {
                    if !disabled.get_untracked() {
                        sql.set(event_target::<web_sys::HtmlTextAreaElement>(&event).value());
                        dismissed.set(false);
                        queue_highlight();
                    }
                }
                on:keydown=on_keydown
                on:keyup=move |event| {
                    if matches!(event.key().as_str(), "ArrowLeft" | "ArrowRight" | "Home" | "End" | "PageUp" | "PageDown")
                        || (matches!(event.key().as_str(), "ArrowUp" | "ArrowDown") && menu.with_untracked(|menu| menu.is_none()))
                    {
                        dismissed.set(false);
                        queue_highlight();
                    }
                }
                on:click=move |_| { dismissed.set(false); queue_highlight(); }
                on:select=move |_| queue_highlight()
                on:focus=move |_| { dismissed.set(false); queue_highlight(); }
                on:blur=move |_| { dismissed.set(true); menu.set(None); }
                on:scroll=move |_| sync_scroll()
                on:compositionstart=move |_| { composing.set(true); menu.set(None); }
                on:compositionend=move |_| {
                    composing.set(false);
                    if !disabled.get_untracked() {
                        if let Some(input) = input_ref.get_untracked() {
                            sql.set(input.value());
                        }
                        dismissed.set(false);
                        queue_highlight();
                    }
                }
            ></textarea>
        </div>
        <Portal>
            <Show when=move || menu.with(|menu| menu.is_some())>
                <div class="sql-completion" style=move || position.get()
                    on:mousedown=move |event| event.prevent_default()
                >
                    <div class="sql-completion-list" role="listbox" id=popup_id.get_value()
                        aria-label="SQL suggestions" node_ref=list_ref
                    >
                        {move || {
                            let option_id = option_id.get_value();
                            menu.get().map(|menu| menu.candidates.into_iter().enumerate().map(|(index, candidate)| {
                                view! {
                                    <div class="sql-completion-option" role="option"
                                        id=format!("{option_id}-{index}")
                                        aria-selected=move || (selected.get() == index).to_string()
                                        on:click=move |_| accept(index)
                                    >
                                        <span class="sql-completion-label">{candidate.label}</span>
                                        <span class="sql-completion-detail">{candidate.detail}</span>
                                    </div>
                                }
                            }).collect_view())
                        }}
                    </div>
                    <div class="sql-completion-help">"↑↓ navigate · Tab accept · Esc dismiss"</div>
                </div>
            </Show>
        </Portal>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlighted_parts_preserve_source_and_only_mark_keywords() {
        let sql = "SeLeCt 名称, '<script>FROM</script>' FROM \"where\" -- JOIN\nWHERE 名称 = $$SELECT$$;\n";
        let parts = highlighted_parts(sql);
        assert_eq!(
            parts
                .iter()
                .map(|(text, _)| text.as_str())
                .collect::<String>(),
            sql
        );
        let keywords: Vec<_> = parts
            .iter()
            .filter(|(_, keyword)| *keyword)
            .map(|(text, _)| text.as_str())
            .collect();
        assert_eq!(keywords, ["SeLeCt", "FROM", "WHERE"]);
    }

    #[test]
    fn completion_replaces_only_identifier_including_suffix_and_preserves_unicode() {
        let text = "SELECT '🐾' FROM creatures WHERE typos = 'Beast';";
        let start = text.find("typos").unwrap();
        let context = completion_context(text, byte_to_utf16(text, start + 2)).unwrap();
        assert_eq!(
            replacement_sql(text, &context, "type"),
            "SELECT '🐾' FROM creatures WHERE type = 'Beast';"
        );
    }

    #[test]
    fn highlighted_lines_round_trip_and_key_repeated_lines_uniquely() {
        let sql = "SELECT 1\nSELECT 1\n/* SELECT\nSELECT */ FROM\n";
        let lines = highlighted_lines(sql);
        let text: String = lines
            .iter()
            .flat_map(|(line, _)| line.iter().map(|(text, _)| text.as_str()))
            .collect();
        assert_eq!(text, sql);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].0, lines[1].0);
        assert_eq!((lines[0].1, lines[1].1), (1, 2));
        // Identical text inside a comment is a different line, not a repeat.
        assert!(lines[3]
            .0
            .iter()
            .any(|(text, keyword)| *keyword && text == "FROM"));
        assert!(!lines[3]
            .0
            .iter()
            .any(|(text, keyword)| *keyword && text.contains("SELECT")));
        assert_eq!(highlighted_lines("no newline").len(), 1);
        assert!(highlighted_lines("").is_empty());
    }

    #[test]
    fn empty_and_plain_inputs_do_not_invent_text_or_keywords() {
        assert!(highlighted_parts("").is_empty());
        assert_eq!(
            highlighted_parts("name 'SELECT'\n"),
            vec![("name 'SELECT'\n".into(), false)]
        );
    }
}
