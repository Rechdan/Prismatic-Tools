//! The Windows shell: a system-tray presence that hosts the tool surface inside
//! the themed window. Window plumbing lives in [`crate::window`].

use std::ptr;

use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use windows_reactor::*;
use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows_sys::Win32::System::Threading::CreateMutexW;

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

/// The hosted tool surface: the persistent app header above the loaded widget
/// (`tool-surface`, `app-header`, `widget-runtime` specs). The first widget found
/// in `widgets/` (beside the exe) is loaded once and rendered below the header.
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
    // Which content the right pane shows: the active widget (`false`) or the
    // config placeholder (`true`). The header Configs button toggles it
    // (`app-header`, `main-content-layout` specs); it starts on the widget.
    let (show_config, set_show_config) = cx.use_state(false);
    // Fully-qualified: `use windows_reactor::*` shadows std `Result` with reactor's
    // one-arg alias, so name the two-arg std form explicitly here.
    let slot = cx.use_ref(None::<std::result::Result<widget::LoadedWidget, widget::WidgetError>>);
    if slot.borrow().is_none() {
        *slot.borrow_mut() = Some(widget::load_first());
    }
    // The right pane's widget content, plus the loaded widget's name for the nav
    // column. A broken/absent widget only replaces the right pane and yields no
    // nav entry; the header and nav heading stay.
    let (content, nav_name): (Element, Option<String>) = match &*slot.borrow() {
        Some(Ok(w)) => (w.render(&set_tick, tick), Some(w.name().to_string())),
        // A benign "no widget" state reads as a plain notice; a real failure is
        // marked as an error. Neither yields a nav entry.
        Some(Err(e)) if e.is_empty_notice() => (text_block(e.to_string()).into(), None),
        Some(Err(e)) => (text_block(format!("⚠ {e}")).into(), None),
        None => (text_block(String::new()).into(), None),
    };

    // Header row: app name on the left, action buttons pinned to the far right. A
    // two-column grid — a star-sized first column (the name) eats the free space,
    // an auto-sized second column (the buttons) hugs the right edge — gives the
    // space-between layout (`hstack` only left-packs). The GitHub `HyperlinkButton`
    // opens the repo page in the default browser; "Configs" activates the config
    // view in the right pane (`app-header` spec) — it selects the config view
    // rather than toggling; the widget view is reached back by clicking the
    // widget's entry in the nav column.
    let header: Element = grid((
        text_block("Prismatic Tools")
            .font_size(20.0)
            .bold()
            .grid_column(0)
            .vertical_alignment(VerticalAlignment::Center),
        hstack((
            button("Configs").on_click(set_show_config.setter(true)),
            HyperlinkButton::new("GitHub")
                .navigate_uri("https://github.com/Rechdan/Prismatic-Tools"),
        ))
        .spacing(8.0)
        .grid_column(1),
    ))
    .columns([GridLength::Star(1.0), GridLength::Auto])
    .column_spacing(8.0)
    .into();

    // Left navigation column: a "Tools" heading plus the loaded widget's name as a
    // clickable entry that activates the widget view (selecting it also brings the
    // user back from the config view). Per-widget custom rendering of the entry is
    // future work. With no widget loaded the heading stands alone
    // (`main-content-layout` spec).
    let mut nav_children: Vec<Element> = vec![text_block("Tools").bold().into()];
    if let Some(name) = nav_name {
        nav_children.push(button(name).on_click(set_show_config.setter(false)).into());
    }
    let nav = vstack(nav_children).spacing(8.0).grid_column(0);

    // Right container: the config placeholder when the config view is active,
    // otherwise the active widget.
    let right: Element = (if show_config { config_view() } else { content }).grid_column(1);

    // Main region: a persistent two-pane grid — a fixed 200-DIP nav column beside
    // a star-sized right container that fills the rest (`main-content-layout` spec).
    let main: Element = grid((nav, right))
        .columns([GridLength::Pixel(200.0), GridLength::Star(1.0)])
        .column_spacing(12.0)
        .into();

    // Window body: header on top (auto height), main filling the rest (star row).
    // The inset from the window border is a `.margin(..)` on the body — WinUI
    // `Grid` has no Padding, so the old outer-column padding becomes a body margin
    // (`app-header` inset requirement).
    grid((header.grid_row(0), main.grid_row(1)))
        .rows([GridLength::Auto, GridLength::Star(1.0)])
        .row_spacing(12.0)
        .margin(16.0)
        .into()
}

/// The config view: a placeholder shown in the main region's right container when
/// the header Configs toggle is on. Reads and writes nothing — a visual stand-in
/// until a real config surface lands (`main-content-layout` spec).
fn config_view() -> Element {
    vstack((
        text_block("Configs").font_size(20.0).bold(),
        text_block("Configuration coming soon."),
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
