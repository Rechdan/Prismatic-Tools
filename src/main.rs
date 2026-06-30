// Prismatic Tools — a Windows 11 system-tray app hosting individual tools in a
// Mica-backed, OS-themed WinUI 3 window (via the pure-Rust `windows-reactor`).
//
// The product is Windows-only. The dev host is WSL2 Linux, so the actual app
// lives behind `cfg(windows)` and is built with `--target x86_64-pc-windows-gnu`
// (see `scripts/winrun.py`). On the Linux host, `main` is a stub so `cargo build`
// / `cargo test` / `yarn dev` stay green.

#[cfg(windows)]
mod shell;
#[cfg(windows)]
mod window;

#[cfg(windows)]
fn main() -> windows_reactor::Result<()> {
    shell::run()
}

#[cfg(not(windows))]
fn main() {
    eprintln!(
        "Prismatic Tools targets Windows. Build/run with:\n  \
         python3 scripts/winrun.py   (cargo build --target x86_64-pc-windows-gnu + stage + run)"
    );
}
