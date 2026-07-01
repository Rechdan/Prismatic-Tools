---
name: audit-docs
description: Relentlessly audit CLAUDE.md and openspec/config.yaml for staleness, gaps, and structure/bloat, then apply fixes through a review gate.
disable-model-invocation: true
---

Audit `CLAUDE.md` and `openspec/config.yaml` **relentlessly**, then apply fixes through a review gate.

A doc is a set of *claims*; the codebase is *ground truth*. Every claim is stale until the code confirms it. Never grade a doc by reading the doc — open the file, run the command, or grep the symbol it names. A claim you remember being true is unverified. The structure axis (below) inherits the same discipline: a finding must point to a *matched trigger*, never a feeling.

Two targets:

- **CLAUDE.md** — the agent instructions. Gets all three axes.
- **openspec/config.yaml** — the `context:` and per-artifact `rules:` fed to AI when generating proposals. `context:` is meant to track the stack; an empty or outdated `context:` is the gap. Gets stale + gap only (a `context:` list has no prose to restructure).

Three axes of finding:

- **Stale** (both targets) — the doc asserts something the code contradicts or no longer supports.
- **Gap** (both targets) — the code carries something load-bearing the doc never mentions. Two refinements: a fact covered by a doc CLAUDE.md **links** (e.g. `docs/reactor-notes.md`) is **not** a gap — it's intentional offload, don't flag it. And placement follows the **auto-load asymmetry**: CLAUDE.md auto-loads every session, sibling `docs/` files do not — so break-your-build / sacred facts must land in CLAUDE.md itself, while genuine deep-dives *should* be offloaded to a linked doc and pointed to.
- **Structure / bloat** (CLAUDE.md / prose only) — the doc is accurate but hard to use. This axis is a fixed checklist of named patterns; a finding must cite a matched trigger:
  - **narrative-changelog** — trigger: "added later" / "came later still" / "Most recently" / inline archive-or-commit citations threaded through prose. Fix: strip to current-state description + one provenance pointer (git history + the archive already hold the sequence).
  - **wall-of-text** — trigger: a paragraph past ~4 sentences packing multiple independent facts. Fix: break into bullets or sub-sections.
  - **descriptive-enumeration** — trigger: symbol / const / message names listed as pure narration, one grep from the source. Fix: compress to behavior + cite the source file. (See the keep-rule — a *corrective* symbol name is not enumeration.)
  - **redundancy** — trigger: the same fact stated in ≥2 places. Fix: single-home it (one canonical location; other spots reference or drop).
  - **section-overlap** — trigger: two sections describing the same subject. Fix: split by question (e.g. *what/why* vs *how/where*) so each heading earns its keep.

  **Keep-rule (fires while editing, per sentence).** A structure fix cuts text; the keep-rule guards what must not be cut. A symbol or detail survives compression if it is a **correction** ("do X, not Y" — e.g. "min size via `App::inner_constraints`, *not* a `WM_GETMINMAXINFO` subclass"), a **non-obvious entry point**, or a **because-clause** explaining a non-obvious constraint ("gnu, not MSVC, *because no `link.exe`*"). Only pure narration is compressible. Stripping a correction or a because-clause makes the doc *worse* — that is the failure mode this axis exists to prevent.

## Steps

1. **Inventory every claim (and, for CLAUDE.md, every structure trigger).** Read both targets top to bottom and extract each atomic, checkable assertion: every command, path, filename, named symbol, version pin, config key, and stated behavior. For config.yaml, treat each line of `context:` and each `rules:` entry as a claim. While reading CLAUDE.md, also flag every structure-pattern match from the checklist above. *Done when* every non-prose line of both files maps to at least one extracted claim, and CLAUDE.md's structure triggers are all noted.

2. **Fan out verification (stale pass).** Split the claims into chunks and dispatch each to a parallel `cavecrew-investigator` subagent: each reaches the actual source — the path, the command in `package.json`/`scripts`/`Cargo.toml`, the symbol in `src/`, the pin in `.nvmrc`/`Cargo.toml` — and returns a verdict per claim: `confirmed`, `stale` (with contradicting evidence), or `unverifiable`. When git history runs deeper than the initial commit, hand investigators the files changed since each doc's last commit so they hit the suspect regions first. *Done when* every claim from step 1 carries a verdict — none graded from memory.

3. **Hunt gaps (coverage pass).** Fan out a walk of the live repo — top-level dirs, entry points, build/config files, scripts, dependency manifests, test layout — and for each area decide: documented (in CLAUDE.md *or* a doc it links), or a load-bearing omission. A fact covered by a linked doc is not a gap. When flagging a real gap, decide placement by the auto-load asymmetry: a break-your-build / sacred fact belongs in CLAUDE.md itself; a deep-dive belongs in a linked doc with a pointer from CLAUDE.md. Files added since a doc's last commit are prime gap suspects (only when history runs deeper than the initial commit). For config.yaml, derive the current stack from the codebase and diff it against `context:`. *Done when* every top-level area is accounted for as documented or a deliberate skip.

4. **Assemble the brief, then stop.** Collect every finding into one report, ordered most-severe first: **stale** (claim → contradicting evidence → correction), **gaps** (undocumented truth → why it matters → where it should land), **structure** (matched trigger → prescribed fix; CLAUDE.md only), **unverifiable** (flagged for the user, never guessed), and the **confirmed-correct** sections to preserve. Then, separately, enumerate the **sacred-facts list**: the load-bearing, break-your-build atoms that must survive **verbatim** through any rewrite (the corrections, the exit codes, the version pins, the toolchain quirks) — this is distinct from "confirmed-correct sections": that bucket says *don't re-verify*, the sacred list says *don't drop while restructuring*. Present it all and **wait for approval** — make no edits until the user signs off.

5. **Apply, branched by target.**
   - **CLAUDE.md** → **direct surgical edit** against the live file, driven by the approved brief (all three axes) and the sacred-facts list. Do **not** use `/init`: it regenerates from a fresh scan, silently dropping hand-won gotchas the scan won't rediscover and imposing its own layout over the structure fixes. Edit in place so every correction, gap-fill, and structure fix lands while every sacred atom is preserved word-for-word. Gate: `git diff` for the human review, plus a grep confirming **every** sacred atom still appears in the rewrite before you finish.
   - **openspec/config.yaml** → apply targeted edits directly: fill or refresh `context:` from the stack derived in step 3, and correct any stale `rules:`. (Already direct-edit — the CLAUDE.md `/init` change does not touch this branch.)
