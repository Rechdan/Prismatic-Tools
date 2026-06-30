//! Window control: the reactor window entrypoint plus Win32 show/hide for the
//! single top-level window. Isolating reactor's window API and the HWND plumbing
//! here localizes churn from the experimental `windows-reactor` crate.

use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

use windows_reactor::*;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CallWindowProcW, FindWindowW, GetClassNameW, GetWindowTextW, IsWindowVisible,
    SetForegroundWindow, SetWindowLongPtrW, SetWindowsHookExW, ShowWindow, UnhookWindowsHookEx,
    CBT_CREATEWNDW, GWLP_WNDPROC, HCBT_ACTIVATE, HCBT_CREATEWND, HHOOK, SWP_HIDEWINDOW,
    SWP_SHOWWINDOW, SW_HIDE, SW_SHOWNORMAL, WH_CBT, WINDOWPOS, WM_CLOSE, WM_WINDOWPOSCHANGING,
    WNDPROC,
};

/// Window title — set by reactor shortly after window creation; used both to
/// locate the top-level HWND and to identify it in the startup hook.
const WINDOW_TITLE: &str = "Prismatic Tools";

/// WinAppSDK desktop main-window class. The startup hook matches this at window
/// creation to claim our window before reactor paints it.
const WINUI_CLASS: &str = "WinUIDesktopWin32WindowClass";

/// Cached top-level HWND of our WinUI window. Reactor's own `AppWindow` bindings
/// are `pub(crate)`, so we drive visibility through the Win32 HWND.
static WINDOW_HWND: AtomicIsize = AtomicIsize::new(0);

/// Original window proc, saved when we subclass the HWND.
static ORIG_WNDPROC: AtomicIsize = AtomicIsize::new(0);

/// Handle of the startup CBT hook (as `isize`; `0` == none).
static HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);

/// While true, the subclass forces the window to stay hidden by rewriting
/// `WM_WINDOWPOSCHANGING` (strip `SWP_SHOWWINDOW`, add `SWP_HIDEWINDOW`). Set at
/// window creation, cleared on the first tray reveal. This swallows reactor's
/// startup activation so the window never paints — no launch flash.
static SUPPRESS_SHOW: AtomicBool = AtomicBool::new(false);

/// Guard so the HWND is subclassed exactly once (the startup hook does it; the
/// `install_close_to_tray` fallback is then a no-op).
static SUBCLASSED: AtomicBool = AtomicBool::new(false);

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Locate and cache the window HWND by title. Call once, at first render.
///
/// Fallback path: the startup CBT hook ([`arm_startup_hook`]) normally caches the
/// HWND at creation, so this is a no-op then. Only if the hook never matched do
/// we fall back to a title lookup (the flash suppression is then lost, degrading
/// to the historical brief flash rather than a broken window).
pub fn capture_hwnd() {
    if WINDOW_HWND.load(Ordering::SeqCst) != 0 {
        return;
    }
    let hwnd = unsafe { FindWindowW(ptr::null(), wide(WINDOW_TITLE).as_ptr()) };
    WINDOW_HWND.store(hwnd as isize, Ordering::SeqCst);
}

fn cached() -> HWND {
    WINDOW_HWND.load(Ordering::SeqCst) as HWND
}

/// Read a window's class name (empty on failure).
fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    if n <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..n as usize])
}

/// Read a window's title text (empty on failure).
fn window_text(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    if n <= 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..n as usize])
}

/// Hide the window — used at startup so nothing shows until the tray reveals it.
pub fn hide() {
    let hwnd = cached();
    if !hwnd.is_null() {
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}

/// Show if hidden, hide if visible — the tray's primary action.
pub fn toggle() {
    let hwnd = cached();
    if hwnd.is_null() {
        return;
    }
    unsafe {
        if IsWindowVisible(hwnd) != 0 {
            ShowWindow(hwnd, SW_HIDE);
        } else {
            // First/any reveal: stop suppressing shows, then show. Clearing the
            // flag lets the `WM_WINDOWPOSCHANGING` rewrite below pass through so
            // the window can actually become visible.
            SUPPRESS_SHOW.store(false, Ordering::SeqCst);
            ShowWindow(hwnd, SW_SHOWNORMAL);
            SetForegroundWindow(hwnd);
        }
    }
}

/// Subclass window proc. Two jobs:
/// 1. While [`SUPPRESS_SHOW`] is set, rewrite `WM_WINDOWPOSCHANGING` to keep the
///    window hidden — this swallows reactor's startup activation (no flash).
/// 2. Swallow `WM_CLOSE` by hiding to the tray (the process exits only via the
///    tray "Exit" item).
/// All other messages forward to the original proc via `CallWindowProcW`.
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_WINDOWPOSCHANGING && SUPPRESS_SHOW.load(Ordering::SeqCst) {
        let wp = lparam as *mut WINDOWPOS;
        if !wp.is_null() {
            unsafe {
                (*wp).flags = ((*wp).flags & !SWP_SHOWWINDOW) | SWP_HIDEWINDOW;
            }
        }
    }
    if msg == WM_CLOSE {
        unsafe { ShowWindow(hwnd, SW_HIDE) };
        return 0;
    }
    let orig = ORIG_WNDPROC.load(Ordering::SeqCst);
    let orig: WNDPROC = unsafe { std::mem::transmute::<isize, WNDPROC>(orig) };
    unsafe { CallWindowProcW(orig, hwnd, msg, wparam, lparam) }
}

/// Subclass the HWND once, storing the original proc for forwarding. Idempotent.
fn subclass(hwnd: HWND) {
    if hwnd.is_null() || SUBCLASSED.swap(true, Ordering::SeqCst) {
        return;
    }
    let orig =
        unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, wnd_proc as *const () as usize as isize) };
    ORIG_WNDPROC.store(orig, Ordering::SeqCst);
}

/// Fallback subclass installer for the close-to-tray behavior, run from the
/// render effect. A no-op when the startup hook already subclassed.
pub fn install_close_to_tray() {
    subclass(cached());
}

/// Adopt `hwnd` as our window: cache it, start suppressing shows, and subclass
/// it. Called the moment the hook first identifies our window.
fn adopt_window(hwnd: HWND) {
    WINDOW_HWND.store(hwnd as isize, Ordering::SeqCst);
    SUPPRESS_SHOW.store(true, Ordering::SeqCst);
    subclass(hwnd);
}

/// Thread-local CBT hook. Identifies our top-level window as early as possible —
/// at `HCBT_CREATEWND` by window class (before any paint, so the suppression is
/// in place when reactor activates), with an `HCBT_ACTIVATE` title backstop in
/// case the class ever changes — then adopts it and unhooks itself (one-shot).
///
/// The create path is flash-free; the activate backstop runs later (the window
/// may already be showing) so it degrades to capturing the HWND for close-to-tray
/// rather than guaranteeing no flash.
unsafe extern "system" fn cbt_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let unclaimed = WINDOW_HWND.load(Ordering::SeqCst) == 0;
    if unclaimed && code == HCBT_CREATEWND as i32 {
        let hwnd = wparam as HWND;
        let info = lparam as *const CBT_CREATEWNDW;
        if !hwnd.is_null() && !info.is_null() {
            let cs = unsafe { (*info).lpcs };
            let top_level = !cs.is_null() && unsafe { (*cs).hwndParent }.is_null();
            if top_level && class_name(hwnd) == WINUI_CLASS {
                adopt_window(hwnd);
                unhook_startup();
            }
        }
    } else if unclaimed && code == HCBT_ACTIVATE as i32 {
        let hwnd = wparam as HWND;
        if !hwnd.is_null() && window_text(hwnd) == WINDOW_TITLE {
            adopt_window(hwnd);
            unhook_startup();
        }
    }
    unsafe { CallNextHookEx(ptr::null_mut(), code, wparam, lparam) }
}

/// Install the startup CBT hook on the current (UI) thread. Call once, right
/// before reactor's message loop, so it is live when reactor creates the window.
fn arm_startup_hook() {
    let tid = unsafe { GetCurrentThreadId() };
    let hook = unsafe { SetWindowsHookExW(WH_CBT, Some(cbt_proc), ptr::null_mut(), tid) };
    HOOK_HANDLE.store(hook as isize, Ordering::SeqCst);
}

/// Remove the startup CBT hook if still installed (idempotent).
fn unhook_startup() {
    let hook = HOOK_HANDLE.swap(0, Ordering::SeqCst) as HHOOK;
    if !hook.is_null() {
        unsafe {
            UnhookWindowsHookEx(hook);
        }
    }
}

/// Run the Mica-backed, OS-themed window hosting `root`. Blocks on the WinUI
/// message loop. `RequestedTheme::Default` (reactor's default) already follows
/// the OS light/dark setting, so the window matches the OS theme automatically.
///
/// The CBT hook is armed first so it can subclass + suppress-show the window at
/// creation, before reactor's activation paints it — that is what makes launch
/// flash-free.
pub fn run(root: fn(&mut RenderCx) -> Element) -> Result<()> {
    arm_startup_hook();
    let result = App::new()
        .title(WINDOW_TITLE)
        .backdrop(Backdrop::Mica)
        .render(root);
    unhook_startup();
    result
}
