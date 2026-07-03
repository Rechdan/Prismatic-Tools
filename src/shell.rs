//! The Windows shell: a system-tray presence that hosts the tool surface inside
//! the themed window. Window plumbing lives in [`crate::window`].

use std::ptr;
use std::time::Duration;

use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use windows_reactor::*;
use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows_sys::Win32::System::Threading::CreateMutexW;

use crate::home;
use crate::widget;
use crate::window;

/// Session-scoped mutex name guarding against a second running instance.
const INSTANCE_MUTEX: &str = "Local\\PrismaticTools.SingleInstance";

/// Acquire the single-instance mutex. Returns the held handle on first launch, or
/// `None` if another instance already owns it (the caller should exit). The
/// handle is kept alive for the process lifetime; the OS releases it on exit.
fn acquire_single_instance() -> Option<HANDLE> {
    let name: Vec<u16> = INSTANCE_MUTEX
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let handle = unsafe { CreateMutexW(ptr::null(), 1, name.as_ptr()) };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        None
    } else {
        Some(handle)
    }
}

/// 16×16 RGBA brand glyph (prism purple) for the tray icon.
fn brand_icon() -> Icon {
    let mut rgba = Vec::with_capacity(16 * 16 * 4);
    for _ in 0..(16 * 16) {
        rgba.extend_from_slice(&[150, 80, 220, 255]);
    }
    Icon::from_rgba(rgba, 16, 16).expect("brand icon")
}

/// Which view the right container shows (`main-content-layout` spec). `Home` (the
/// rendered README) is the default landing view; the widget and config views are
/// reached from the nav widget card and the nav Configs entry, respectively. The
/// active view's nav entry carries a selection highlight.
#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Home,
    Widget,
    Config,
}

/// The hosted tool surface: a single left navigation sidebar beside the selected
/// view (`tool-surface`, `main-content-layout`, `widget-runtime` specs). The first
/// widget found in `widgets/` (beside the exe) is loaded once; its card lives in the
/// sidebar and its UI renders in the right container when the widget view is active.
fn tool_surface(cx: &mut RenderCx) -> Element {
    // No window on launch (`tray-presence` spec): the startup CBT hook already
    // claimed the window at creation — subclassing it to suppress shows, so
    // reactor's activation never painted it (no flash). So here `capture_hwnd()`
    // and `install_close_to_tray()` are no-ops, and `hide()` just reinforces the
    // hidden state. The user reveals it from the tray. If the hook ever misses,
    // these fall back to a title lookup + subclass and we degrade to the
    // historical brief flash rather than a broken window.
    cx.use_effect((0u8,), || {
        window::capture_hwnd();
        window::install_close_to_tray();
        window::hide();
    });

    // The hosted widget persists across renders in a `use_ref`, loaded once on the
    // first render. `tick` is the re-render trigger a widget callback bumps after
    // mutating its Lua state (the widget's own state lives inside its VM).
    let (tick, set_tick) = cx.use_state(0_u32);
    // Which content the right pane shows: the home view (the rendered README, the
    // default), the active widget, or the config placeholder. The nav Home entry,
    // the nav widget card, and the nav Configs entry each select one
    // (`home-screen`, `main-content-layout` specs).
    let (view, set_view) = cx.use_state(View::Home);
    // Hover state for the nav card below; drives its animated highlight fill.
    let (hovered, set_hovered) = cx.use_state(false);
    // Fully-qualified: `use windows_reactor::*` shadows std `Result` with reactor's
    // one-arg alias, so name the two-arg std form explicitly here.
    let slot = cx.use_ref(None::<std::result::Result<widget::LoadedWidget, widget::WidgetError>>);
    if slot.borrow().is_none() {
        *slot.borrow_mut() = Some(widget::load_first());
    }
    // The right pane's widget content, plus the nav card (resolved style + preview
    // content) for the left column. A broken/absent widget only replaces the right
    // pane and yields no nav card; the header and nav heading stay.
    let (content, nav_card): (Element, Option<(widget::BorderStyle, Element)>) =
        match &*slot.borrow() {
            Some(Ok(w)) => {
                // The widget's custom preview when it ships a `nav`, else its name
                // on the default card.
                let card = w
                    .nav_render()
                    .map(|widget::NavCard { style, content }| (style, content))
                    .unwrap_or_else(|| {
                        (
                            widget::BorderStyle::default_card(),
                            text_block(w.name().to_string()).into(),
                        )
                    });
                (w.render(&set_tick, tick), Some(card))
            }
            // A benign "no widget" state reads as a plain notice; a real failure is
            // marked as an error. Neither yields a nav card.
            Some(Err(e)) if e.is_empty_notice() => (text_block(e.to_string()).into(), None),
            Some(Err(e)) => (text_block(format!("⚠ {e}")).into(), None),
            None => (text_block(String::new()).into(), None),
        };

    // A navigation entry with a view-driven selection highlight: a real
    // (keyboard-focusable) `.subtle()` button — transparent at rest — layered over a
    // soft selection fill whose opacity is driven declaratively by the active `view`.
    // No toggle/checked control state, so re-selecting the already-active entry is a
    // harmless no-op and the highlight never desyncs (a controlled `ToggleButton`
    // would deselect itself on re-click). Grid children overlap at cell (0,0); the
    // button is the later child, so it sits above the fill and receives clicks
    // (`main-content-layout` selection requirement).
    let nav_button = |label: &str, target: View| -> Element {
        let selection_fill = border(text_block(""))
            .background(ThemeRef::ControlFill)
            .corner_radius(4.0)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch)
            .opacity(if view == target { 1.0 } else { 0.0 })
            .with_opacity_transition(Duration::from_millis(150));
        let btn = button(label)
            .subtle()
            .on_click(set_view.setter(target))
            .horizontal_alignment(HorizontalAlignment::Stretch);
        grid((selection_fill, btn)).into()
    };

    // The loaded widget's card: a full-width clickable tile (tapped `border`)
    // activating the widget view, showing the widget's custom preview or its name.
    // It layers, beneath the content, a persistent selection fill (lit when the
    // widget view is active) and, above that, a transient hover fill — the two
    // compose. No widget → no card (`main-content-layout` spec).
    let nav_card_el: Option<Element> = nav_card.map(|(style, content)| {
        let radius = style.corner_radius().unwrap_or(0.0);
        // Pad the content, not the card frame, so the background layers below span
        // the whole card (only the text is inset).
        let content = match style.padding() {
            Some(p) => content.padding(Thickness::uniform(p)),
            None => content,
        };
        // Selection fill: a ControlFill layer covering the full card, lit when the
        // widget view is active. Distinct brush from the hover's SubtleFill so the
        // two read differently and compose.
        let selection_fill = border(text_block(""))
            .background(ThemeRef::ControlFill)
            .corner_radius(radius)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch)
            .opacity(if view == View::Widget { 1.0 } else { 0.0 })
            .with_opacity_transition(Duration::from_millis(150));
        // Hover highlight: a SubtleFill layer above the selection fill, beneath the
        // content, whose opacity fades in/out (brush color can't tween, so we
        // crossfade opacity). Its radius matches the card's.
        let fill = border(text_block(""))
            .background(ThemeRef::SubtleFill)
            .corner_radius(radius)
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .vertical_alignment(VerticalAlignment::Stretch)
            .opacity(if hovered { 1.0 } else { 0.0 })
            .with_opacity_transition(Duration::from_millis(150));
        let set_enter = set_hovered.clone();
        let set_exit = set_hovered.clone();
        // Host owns behavior (applied last, not Lua-overridable): hover tracking,
        // tap-to-activate the widget view, and full-column stretch. Frame props
        // (background/stroke/radius) go on the outer border; padding moved to
        // content above.
        style
            .apply_frame(border(grid((selection_fill, fill, content))))
            .on_pointer_entered(move |_| set_enter.call(true))
            .on_pointer_exited(move || set_exit.call(false))
            .on_tapped(set_view.setter(View::Widget))
            .horizontal_alignment(HorizontalAlignment::Stretch)
            .into()
    });

    // Pinned app title (nav row 0): a plain, non-clickable brand label, always
    // visible above the scrollable list (`main-content-layout` app-title req).
    let title = text_block("Prismatic Tools").font_size(20.0).bold();

    // Scrollable tool list (nav row 1): the Home entry, the `Tools` heading, and the
    // widget card, wrapped in a vertical `scroll_viewer` so an overlong list scrolls
    // instead of overrunning the pinned bottom group. The `vstack` (a real panel) is
    // the scroll_viewer's sole child. The `Star` grid cell bounds its height.
    let mut tool_children: Vec<Element> = vec![
        nav_button("Home", View::Home),
        text_block("Tools").bold().into(),
    ];
    if let Some(card) = nav_card_el {
        tool_children.push(card);
    }
    let tool_list = scroll_viewer(vstack(tool_children).spacing(8.0));

    // Pinned bottom action group (nav row 2): the Configs entry (a view, highlighted)
    // above the GitHub external link (never highlighted). GitHub opens the repo page
    // in the default browser (`main-content-layout` bottom-group reqs).
    let bottom = vstack((
        nav_button("Configs", View::Config),
        HyperlinkButton::new("GitHub")
            .navigate_uri("https://github.com/Rechdan/Prismatic-Tools")
            .horizontal_alignment(HorizontalAlignment::Stretch),
    ))
    .spacing(8.0);

    // Reload action for the config view: rebuild the widget from disk (a fresh VM,
    // so in-memory state resets), then bump the tick to re-render. The label
    // pluralizes by the installed-widget count; reloading leaves the active view
    // unchanged (`tool-surface`, `main-content-layout` specs).
    let reload_label = if widget::count() == 1 {
        "Reload widget"
    } else {
        "Reload widgets"
    };
    let reload = {
        let slot = slot.clone();
        let set_tick = set_tick.clone();
        let next = tick.wrapping_add(1);
        move || {
            *slot.borrow_mut() = Some(widget::load_first());
            set_tick.call(next);
        }
    };

    // Right container: exactly one active view — the home README (default), the
    // active widget, or the config view (which hosts the reload action).
    let right: Element = match view {
        View::Home => home::view(),
        View::Widget => content,
        View::Config => config_view(reload_label, reload),
    }
    .grid_column(1);

    // Navigation column (nav grid): pinned title, scrollable list, pinned bottom
    // actions — an `Auto`/`Star`/`Auto` three-row grid whose `Star` middle absorbs
    // the free height, keeping the title and the actions pinned to the column's top
    // and bottom edges (`main-content-layout` spec).
    let nav: Element = grid((
        title.grid_row(0),
        tool_list.grid_row(1),
        bottom.grid_row(2),
    ))
    .rows([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
    .row_spacing(8.0)
    .grid_column(0)
    .into();

    // Window body: the two-pane grid fills the whole padded window with no header —
    // a fixed 200-DIP nav column beside a star-sized right container. The inset from
    // the window border is a `.margin(..)` on the body (WinUI `Grid` has no Padding).
    grid((nav, right))
        .columns([GridLength::Pixel(200.0), GridLength::Star(1.0)])
        .column_spacing(12.0)
        .margin(16.0)
        .into()
}

/// The config view: a placeholder shown in the main region's right container when
/// the config view is active. Beyond the placeholder text it hosts the
/// reload-widget action (`main-content-layout`, `tool-surface` specs); it reads and
/// writes no settings. `reload_label` is pluralized by the installed-widget count.
fn config_view(reload_label: &str, on_reload: impl Fn() + 'static) -> Element {
    vstack((
        text_block("Configs").font_size(20.0).bold(),
        text_block("Configuration coming soon."),
        button(reload_label).on_click(on_reload),
    ))
    .spacing(8.0)
    .into()
}

/// Run the shell: install the tray icon, then enter the WinUI message loop.
///
/// The tray is created on the main thread *before* the window's message loop
/// starts; reactor then pumps both the WinUI window and the tray's hidden window
/// (single thread — coexistence proven in the Phase 0 spike). The `MenuEvent`
/// handler therefore runs on the UI thread, where the Win32 calls are valid.
pub fn run() -> Result<()> {
    // Single-instance guard: if the mutex already exists, another instance owns
    // the tray — exit cleanly before building a duplicate icon. `_instance` is
    // held to the end of `run()` (the process lifetime) so the mutex stays owned.
    let _instance = match acquire_single_instance() {
        Some(handle) => handle,
        None => return Ok(()),
    };

    let menu = Menu::new();
    let quit = MenuItem::new("Exit", true, None);
    menu.append(&quit).expect("append quit");
    let quit_id = quit.id().clone();

    // Held until the process exits (kept alive past the message loop below).
    // `with_menu_on_left_click(false)`: a left-click is delivered as a
    // `TrayIconEvent` instead of popping the menu; right-click still opens it.
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip("Prismatic Tools")
        .with_icon(brand_icon())
        .build()
        .expect("build tray icon");

    MenuEvent::set_event_handler(Some(move |ev: MenuEvent| {
        if ev.id == quit_id {
            std::process::exit(0);
        }
    }));

    // Left-click toggles the window. A left-click emits `Click` on both button
    // down and up; match `Up` only so a single click toggles exactly once.
    TrayIconEvent::set_event_handler(Some(|ev: TrayIconEvent| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = ev
        {
            window::toggle();
        }
    }));

    window::run(tool_surface)
}
