//! `home-screen`: render the bundled project README as the app's home view.
//!
//! The README is compiled into the exe (`include_str!`), parsed with
//! `pulldown-cmark`, and its **block-level** markdown mapped onto reactor
//! elements. Inline runs (bold/italic/inline-code/links) are flattened to plain
//! text: reactor's backend renders no per-run inline styling and `RichTextHyperlink`
//! does not navigate (see `docs/reactor-notes.md`), so a plain `text_block` per
//! block is the honest ceiling — and this README has no inline bold to preserve.
//! The whole document is wrapped in a vertical `scroll_viewer`. Rendering is
//! fault-tolerant: an unhandled construct degrades to readable text, never panics
//! (a release build aborts on panic).

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use windows_reactor::*;

/// The project README, bundled at build time (no runtime file dependency). The
/// `include_str!` edge makes a rebuild pick up README edits; `dev:app` watches it.
const README: &str = include_str!("../README.md");

/// The home view: the rendered README inside a vertical scroll viewer, sized to
/// the right container (a Grid star cell bounds its height, so it scrolls).
pub fn view() -> Element {
    scroll_viewer(render_markdown(README)).into()
}

/// A list being assembled: its next ordered index (or `None` when bulleted) and
/// the item rows collected so far.
struct ListFrame {
    ordered_next: Option<u64>,
    items: Vec<Element>,
}

/// Parse `src` and fold its block-level markdown into a top-level `vstack` of
/// block elements. The `vstack` (a real panel, never a bare fragment) is the
/// single child handed to the caller's `scroll_viewer`.
fn render_markdown(src: &str) -> Element {
    let mut out: Vec<Element> = Vec::new();
    // Inline text accumulator for the current leaf block (paragraph / heading /
    // list item). Inline runs concatenate into this; markers are ignored.
    let mut text = String::new();
    // Set while inside a heading (drives its type-ramp size on End).
    let mut heading: Option<HeadingLevel> = None;
    // `Some` while inside a fenced/indented code block (collects raw code text).
    let mut code: Option<String> = None;
    // Blockquote nesting depth (>0 → the current paragraph renders set-off).
    let mut quote_depth: usize = 0;
    // List-item nesting depth (>0 → paragraph ends feed the item, not `out`).
    let mut item_depth: usize = 0;
    // Open lists (supports nesting; the README uses a single level).
    let mut lists: Vec<ListFrame> = Vec::new();

    for ev in Parser::new(src) {
        match ev {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    text.clear();
                    heading = Some(level);
                }
                Tag::Paragraph => {
                    if item_depth == 0 {
                        text.clear();
                    }
                }
                Tag::CodeBlock(_) => code = Some(String::new()),
                Tag::List(start) => lists.push(ListFrame {
                    ordered_next: start,
                    items: Vec::new(),
                }),
                Tag::Item => {
                    item_depth += 1;
                    text.clear();
                }
                Tag::BlockQuote(_) => quote_depth += 1,
                // Emphasis / Strong / Strikethrough / Link and any other inline
                // container: ignored — their text still accumulates via Event::Text.
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(_) => {
                    out.push(heading_block(heading.take(), &text));
                    text.clear();
                }
                TagEnd::Paragraph => {
                    // Top-level (and blockquote) paragraphs flush a block; a
                    // paragraph inside a list item is folded into the item instead.
                    if item_depth == 0 {
                        out.push(paragraph_block(&text, quote_depth > 0));
                        text.clear();
                    }
                }
                TagEnd::CodeBlock => out.push(code_block(&code.take().unwrap_or_default())),
                TagEnd::Item => {
                    item_depth = item_depth.saturating_sub(1);
                    if let Some(frame) = lists.last_mut() {
                        let marker = match &mut frame.ordered_next {
                            Some(n) => {
                                let m = format!("{n}.");
                                *n += 1;
                                m
                            }
                            None => "•".to_string(),
                        };
                        frame.items.push(item_row(&marker, &text));
                    }
                    text.clear();
                }
                TagEnd::List(_) => {
                    if let Some(frame) = lists.pop() {
                        out.push(vstack(frame.items).spacing(4.0).into());
                    }
                }
                TagEnd::BlockQuote(_) => quote_depth = quote_depth.saturating_sub(1),
                _ => {}
            },
            // Inside a code block, raw text (with embedded newlines) goes to the
            // code buffer; otherwise it feeds the current inline accumulator.
            Event::Text(t) => match code.as_mut() {
                Some(c) => c.push_str(&t),
                None => text.push_str(&t),
            },
            // Inline code: flattened into the surrounding text (no monospace run).
            Event::Code(t) => text.push_str(&t),
            Event::SoftBreak => match code.as_mut() {
                Some(c) => c.push('\n'),
                None => text.push(' '),
            },
            Event::HardBreak => match code.as_mut() {
                Some(c) => c.push('\n'),
                None => text.push('\n'),
            },
            Event::Rule => out.push(rule()),
            // HTML, footnotes, task markers, etc.: ignored (degrade to nothing);
            // any text they carry arrives as Event::Text and is still rendered.
            _ => {}
        }
    }

    vstack(out).spacing(10.0).into()
}

/// A heading, sized by level via the WinUI type ramp; wrapping + selectable.
fn heading_block(level: Option<HeadingLevel>, text: &str) -> Element {
    let tb = match level {
        Some(HeadingLevel::H1) => title(text),
        Some(HeadingLevel::H2) => subtitle(text),
        Some(HeadingLevel::H3) => body_large(text).bold(),
        _ => body_strong(text),
    };
    tb.wrap().selectable().into()
}

/// A body paragraph; when inside a blockquote it is set off (dimmed + indented).
fn paragraph_block(text: &str, quoted: bool) -> Element {
    let tb = body(text).wrap().selectable();
    if quoted {
        tb.foreground(ThemeRef::SecondaryText)
            .margin(Thickness {
                left: 16.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            })
            .into()
    } else {
        tb.into()
    }
}

/// One list-item row: a fixed-width marker column beside the wrapping item text.
fn item_row(marker: &str, text: &str) -> Element {
    hstack((
        body(marker).width(20.0),
        body(text).wrap().selectable(),
    ))
    .spacing(6.0)
    .into()
}

/// A fenced/indented code block: monospaced text (block-level `.font_family`,
/// which the backend applies) inside a subtle, padded, rounded container.
fn code_block(code: &str) -> Element {
    border(
        text_block(code.trim_end_matches('\n'))
            .font_family("Consolas")
            .font_size(13.0)
            .wrap()
            .selectable(),
    )
    .background(ThemeRef::SubtleFill)
    .padding(Thickness::uniform(10.0))
    .corner_radius(6.0)
    .into()
}

/// A thematic break (`---`): a thin, full-width divider line.
fn rule() -> Element {
    border(text_block(""))
        .background(ThemeRef::DividerStroke)
        .height(1.0)
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .into()
}
