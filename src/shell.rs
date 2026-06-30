//! The Windows shell: a system-tray presence that hosts the tool surface inside
//! the themed window. Window plumbing lives in [`crate::window`].

use std::ptr;

use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use windows_reactor::*;
use windows_sys::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows_sys::Win32::System::Threading::CreateMutexW;

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

/// The hosted tool surface. For the bootstrap shell this is one hardcoded demo
/// tool that proves the host → tool render/state loop (`tool-surface` spec).
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

    let (count, set_count) = cx.use_state(0_i32);
    // Outer padded column: a persistent app header on top, then the hosted tool
    // (`app-header` spec). The `.padding(..)` insets every child from the window
    // border so nothing sits flush against the Mica edges.
    vstack((
        // Header row: app name on the left, action buttons pinned to the far
        // right. A two-column grid — a star-sized first column (the name) eats
        // the free space, an auto-sized second column (the buttons) hugs the
        // right edge — gives the space-between layout (`hstack` only left-packs).
        // The name lives here now (the tool no longer repeats it). The GitHub
        // `HyperlinkButton` opens the repo page in the default browser;
        // "Configs" is a placeholder until a real config surface lands.
        grid((
            text_block("Prismatic Tools")
                .font_size(20.0)
                .bold()
                .grid_column(0)
                .vertical_alignment(VerticalAlignment::Center),
            hstack((
                button("Configs").on_click(|| {}),
                HyperlinkButton::new("GitHub")
                    .navigate_uri("https://github.com/Rechdan/Prismatic-Tools"),
            ))
            .spacing(8.0)
            .grid_column(1),
        ))
        .columns([GridLength::Star(1.0), GridLength::Auto])
        .column_spacing(8.0),
        text_block("Demo tool — proves the host render/state loop."),
        text_block(format!("clicks: {count}")),
        button("Click me").on_click(move || set_count.call(count + 1)),
    ))
    .spacing(12.0)
    .padding(16.0)
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
