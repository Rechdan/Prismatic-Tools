#!/usr/bin/env python3
"""Build, stage, and run Prismatic Tools on Windows from the WSL2 Linux host.

The product is a WinUI 3 (windows-reactor) app. We cross-build it with the WSL
cargo + mingw toolchain for `x86_64-pc-windows-gnu`, then stage a self-contained
Windows App SDK runtime next to the exe and launch it via WSL interop.

Why not just `windows-reactor-setup` in build.rs? Its `as_self_contained()` only
runs on a Windows build host (it shells out to %SystemRoot% curl/tar and rejects
the plain `gnu` ABI). So we replicate its staging here, from Linux.

Steps:
  1. cargo build --target x86_64-pc-windows-gnu
  2. locate the windows-reactor-setup assets in the cargo git checkout
  3. download + extract the App SDK runtime (cached)
  4. stage Bootstrap.dll + runtime DLLs + <exe>.manifest next to the exe on NTFS
  5. run the exe via interop (unless --no-run)

Usage: python3 scripts/winrun.py [--release] [--no-run]
"""
from __future__ import annotations

import glob
import os
import shutil
import subprocess
import sys
import zipfile

TARGET = "x86_64-pc-windows-gnu"
EXE_NAME = "primatic-tools.exe"
RUNTIME_PKG = "Microsoft.WindowsAppSDK.Runtime"
RUNTIME_VER = "2.1.3"  # keep in sync with windows-reactor-setup
NUGET_URL = "https://www.nuget.org/api/v2/package/{name}/{version}"
MSIX_IN_NUPKG = "tools/MSIX/win10-x64/Microsoft.WindowsAppRuntime.2.msix"

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def sh(cmd: list[str], **kw) -> subprocess.CompletedProcess:
    print("+", " ".join(cmd))
    return subprocess.run(cmd, check=True, **kw)


def find_reactor_setup() -> str:
    """Locate crates/libs/reactor-setup inside the cargo git checkout."""
    pats = os.path.join(
        os.path.expanduser("~/.cargo/git/checkouts"),
        "windows-rs-*", "*", "crates", "libs", "reactor-setup",
    )
    hits = sorted(glob.glob(pats))
    if not hits:
        sys.exit(
            "reactor-setup assets not found in the cargo git checkout.\n"
            "Run a build first so cargo clones microsoft/windows-rs."
        )
    return hits[-1]


def ensure_runtime_extracted(cache: str) -> str:
    """Download + extract the App SDK runtime MSIX; return the extracted dir."""
    os.makedirs(cache, exist_ok=True)
    nupkg = os.path.join(cache, f"{RUNTIME_PKG}.{RUNTIME_VER}.nupkg")
    extract = os.path.join(cache, "msix_extract")
    if os.path.isdir(extract):
        return extract
    if not os.path.isfile(nupkg):
        url = NUGET_URL.format(name=RUNTIME_PKG, version=RUNTIME_VER)
        sh(["curl", "-sL", "-o", nupkg, url])
    with zipfile.ZipFile(nupkg) as z:
        msix_member = next(
            (n for n in z.namelist() if n.lower() == MSIX_IN_NUPKG.lower()), None
        )
        if not msix_member:
            sys.exit(f"{MSIX_IN_NUPKG} not found inside {nupkg}")
        msix_path = z.extract(msix_member, cache)
    os.makedirs(extract, exist_ok=True)
    with zipfile.ZipFile(msix_path) as z:
        z.extractall(extract)
    return extract


def win_run_dir() -> str:
    """A real NTFS directory to stage+run from (UNC \\wsl paths fail to load DLLs)."""
    if os.environ.get("PRISMATIC_RUN_DIR"):
        return os.environ["PRISMATIC_RUN_DIR"]
    out = subprocess.run(
        ["cmd.exe", "/c", "echo %USERPROFILE%"],
        capture_output=True, text=True,
    ).stdout.strip()
    linux = subprocess.run(
        ["wslpath", "-u", out], capture_output=True, text=True
    ).stdout.strip()
    return os.path.join(linux, "PrismaticTools", "run")


def stage(profile: str) -> str:
    setup = find_reactor_setup()
    assets = os.path.join(setup, "assets")
    runtime_extract = ensure_runtime_extracted(
        os.path.join(ROOT, "target", "winstage-cache")
    )

    exe = os.path.join(ROOT, "target", TARGET, profile, EXE_NAME)
    if not os.path.isfile(exe):
        sys.exit(f"exe not found: {exe}")

    dest = win_run_dir()
    if os.path.isdir(dest):
        shutil.rmtree(dest)
    os.makedirs(dest)

    # exe + external SxS manifest (registration-free WinUI 3 activation)
    shutil.copy(exe, dest)
    shutil.copy(os.path.join(assets, "app.manifest"),
                os.path.join(dest, EXE_NAME + ".manifest"))
    # Bootstrap.dll — a STATIC import of the exe, NOT listed in runtime.txt
    shutil.copy(
        os.path.join(setup, "bootstrap", "x64", "Microsoft.WindowsAppRuntime.Bootstrap.dll"),
        dest,
    )
    # runtime DLLs / .pri / locale dirs enumerated by runtime.txt
    wanted = {
        l.strip().lower()
        for l in open(os.path.join(assets, "runtime.txt"))
        if l.strip()
    }
    staged = 0
    for name in os.listdir(runtime_extract):
        if name.lower() in wanted:
            src = os.path.join(runtime_extract, name)
            dst = os.path.join(dest, name)
            if os.path.isdir(src):
                shutil.copytree(src, dst, dirs_exist_ok=True)
            else:
                shutil.copy(src, dst)
            staged += 1
    # widgets/ — Lua tool packages loaded at runtime. The app resolves them
    # relative to the exe, so stage them next to it (dest is wiped each run).
    widgets_src = os.path.join(ROOT, "widgets")
    widgets_note = ""
    if os.path.isdir(widgets_src):
        shutil.copytree(widgets_src, os.path.join(dest, "widgets"))
        widgets_note = " + widgets"
    print(f"staged {staged} runtime entries + bootstrap + manifest{widgets_note} -> {dest}")
    return dest


def main() -> None:
    release = "--release" in sys.argv
    no_run = "--no-run" in sys.argv
    profile = "release" if release else "debug"

    cargo = ["cargo", "build", "--target", TARGET]
    if release:
        cargo.append("--release")
    sh(cargo)

    dest = stage(profile)
    if no_run:
        print(f"staged only (--no-run). Launch: {os.path.join(dest, EXE_NAME)}")
        return
    print("launching...")
    subprocess.run([os.path.join(dest, EXE_NAME)])


if __name__ == "__main__":
    main()
