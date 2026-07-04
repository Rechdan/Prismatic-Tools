//! The Windows shell: a system-tray presence that hosts the tool surface inside
//! the themed window. Window plumbing lives in [`crate::window`].

use std::ptr;

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

/// The hosted tool surface: reactor's native `NavigationView` (a burger-toggled left
/// pane of icon+label items) beside the selected view (`tool-surface`,
/// `main-content-layout`, `widget-runtime` specs). The first widget found in
/// `widgets/` (beside the exe) is loaded once; it appears as a pane item (or, on load
/// failure, a selectable error item) and its UI renders in the content area.
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
    // The active NavigationView selection, as a tag string. Disjoint tag namespaces
    // route the content area (D1/D6): `"home"` → the README home view (the default
    // landing view); `"widget:<id>"` → the widget view (its render, or the load-error
    // text); `"config"` → the config view. Every destination is a real menu item, so
    // each is selectable *and* highlightable by reactor's `select_nav_item_by_tag`.
    // The built-in Settings gear is disabled (`settings_visible(false)`): its
    // selection does not deliver a routable tag through reactor, so Configs is a
    // normal menu item instead.
    //
    // `""` is a deselection artifact, also routed to config: reactor re-applies a
    // changed `menu_items` via `menu.Clear()`, and clearing a *currently-selected*
    // MenuItem makes WinUI fire `SelectionChanged` with a null item → `""`. The only
    // `menu_items` mutator is the reload closure, reachable solely from the config
    // view, so the only item that can be selected when the menu is rebuilt is Configs
    // itself — mapping `""` → config keeps the content on config across a
    // widget-changing reload. Do not add a menu-mutating action reachable from Home or
    // the widget view without revisiting this.
    let (tag, set_tag) = cx.use_state("home".to_string());
    // Fully-qualified: `use windows_reactor::*` shadows std `Result` with reactor's
    // one-arg alias, so name the two-arg std form explicitly here.
    let slot = cx.use_ref(None::<std::result::Result<widget::LoadedWidget, widget::WidgetError>>);
    if slot.borrow().is_none() {
        *slot.borrow_mut() = Some(widget::load_first());
    }

    // Reload action for the config view: rebuild the widget from disk (a fresh VM,
    // so in-memory state resets), then bump the tick to re-render. The label
    // pluralizes by the installed-widget count; reloading leaves the active tag
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

    // Build the pane's menu items and resolve the widget-view content in one pass.
    // Home is always present. On a successful load the widget appears by name with a
    // Document icon; on a real load failure it appears as a selectable error item
    // labeled with its folder id (`Important` icon) whose content is the error text;
    // an absent widget (`NoWidgets`) adds no item (`main-content-layout`, D7).
    let mut menu_items: Vec<NavViewItem> =
        vec![NavViewItem::new("Home").tag("home").icon(Symbol::Home)];
    let widget_content: Element = match &*slot.borrow() {
        Some(Ok(w)) => {
            menu_items.push(
                NavViewItem::new(w.name())
                    .tag(format!("widget:{}", w.id()))
                    .icon(Symbol::Document),
            );
            w.render(&set_tick, tick)
        }
        Some(Err(e)) => match e.id() {
            Some(id) => {
                menu_items.push(
                    NavViewItem::new(id)
                        .tag(format!("widget:{id}"))
                        .icon(Symbol::Important),
                );
                text_block(format!("⚠ {e}")).into()
            }
            // Absent widget: a benign notice in the content area, no pane item.
            None => text_block(e.to_string()).into(),
        },
        None => text_block(String::new()).into(),
    };
    // Configs is a normal, tag-routed menu item (not the built-in gear), placed last.
    // The `Setting` glyph keeps it reading as settings.
    menu_items.push(NavViewItem::new("Configs").tag("config").icon(Symbol::Setting));

    // Route the content area by the current tag (mirrors the old `match view`).
    // `"config"` (the Configs item) or `""` (a reload deselection artifact) → the
    // config view; a `widget:*` tag → the widget content; `"home"` and any other value
    // fall back to the README home view.
    let right_content: Element = match tag.as_str() {
        "config" | "" => config_view(reload_label, reload),
        t if t.starts_with("widget:") => widget_content,
        _ => home::view(),
    };

    // Inset every view from the content-area edges (replacing the deleted body
    // `.margin(16)`). One padded wrapper so the inset can't drift; it MUST stretch to
    // fill the content cell — a size-to-content wrapper would hand `home::view()`'s
    // `scroll_viewer` an unbounded height and the README would stop scrolling
    // (`docs/reactor-notes.md`: a `scroll_viewer` needs a bounded cell).
    let padded = border(right_content)
        .padding(Thickness::uniform(16.0))
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .vertical_alignment(VerticalAlignment::Stretch);

    // Native NavigationView (D1/D2): a Left-mode, burger-toggled pane titled
    // "Prismatic Tools". The built-in Settings gear is hidden (`settings_visible(false)`)
    // — Configs is a normal menu item instead, because the gear's selection does not
    // route through reactor's tag mechanism. `selected_tag` echoes our state
    // (highlighting the active item, Home on first paint); the setter is passed
    // *directly* to `on_selection_changed` (a fresh closure each render would churn the
    // WinUI handler) and stores whatever tag was selected.
    NavigationView::new(menu_items, padded)
        .selected_tag(tag.clone())
        .on_selection_changed(set_tag)
        .pane_display_mode(NavigationViewPaneDisplayMode::Left)
        .pane_toggle_button_visible(true)
        .settings_visible(false)
        .pane_title("Prismatic Tools")
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
