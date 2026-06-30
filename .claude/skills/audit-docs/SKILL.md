---
name: audit-docs
description: Relentlessly audit CLAUDE.md and openspec/config.yaml for staleness and gaps, then apply fixes through a review gate.
disable-model-invocation: true
---

Audit `CLAUDE.md` and `openspec/config.yaml` **relentlessly**, then apply fixes through a review gate.

A doc is a set of *claims*; the codebase is *ground truth*. Every claim is stale until the code confirms it. Never grade a doc by reading the doc — open the file, run the command, or grep the symbol it names. A claim you remember being true is unverified.

Two targets, same treatment:

- **CLAUDE.md** — the agent instructions.
- **openspec/config.yaml** — the `context:` and per-artifact `rules:` fed to AI when generating proposals. `context:` is meant to track the stack; an empty or outdated `context:` is the gap.

Two directions of drift, checked on both:

- **Stale** — the doc asserts something the code contradicts or no longer supports.
- **Gap** — the code carries something load-bearing the doc never mentions.

## Steps

1. **Inventory every claim.** Read both targets top to bottom and extract each atomic, checkable assertion: every command, path, filename, named symbol, version pin, config key, and stated behavior. For config.yaml, treat each line of `context:` and each `rules:` entry as a claim. *Done when* every non-prose line of both files maps to at least one extracted claim.

2. **Fan out verification (stale pass).** Split the claims into chunks and dispatch each to a parallel `cavecrew-investigator` subagent: each reaches the actual source — the path, the command in `package.json`/`scripts`/`Cargo.toml`, the symbol in `src/`, the pin in `.nvmrc`/`Cargo.toml` — and returns a verdict per claim: `confirmed`, `stale` (with contradicting evidence), or `unverifiable`. When git history runs deeper than the initial commit, hand investigators the files changed since each doc's last commit so they hit the suspect regions first. *Done when* every claim from step 1 carries a verdict — none graded from memory.

3. **Hunt gaps (coverage pass).** Fan out a walk of the live repo — top-level dirs, entry points, build/config files, scripts, dependency manifests, test layout — and for each area decide: documented, or a load-bearing omission. Files added since a doc's last commit are prime gap suspects (only when history runs deeper than the initial commit). For config.yaml, derive the current stack from the codebase and diff it against `context:`. *Done when* every top-level area is accounted for as documented or a deliberate skip.

4. **Assemble the brief, then stop.** Collect every finding into one report, ordered most-severe first: **stale** (claim → contradicting evidence → correction), **gaps** (undocumented truth → why it matters), **unverifiable** (flagged for the user, never guessed), and the **confirmed-correct** sections to preserve. Present it and **wait for approval** — make no edits until the user signs off.

5. **Apply, branched by target.**
   - **CLAUDE.md** → invoke the `init` skill (`/init`), passing the approved brief so the rewrite corrects every stale claim, fills every gap, and preserves the sections marked correct.
   - **openspec/config.yaml** → apply targeted edits directly: fill or refresh `context:` from the stack derived in step 3, and correct any stale `rules:`. No /init — it owns CLAUDE.md only.
