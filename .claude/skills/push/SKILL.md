---
name: push
description: Commit only the staged changes with a caveman-commit-style message, then push — pausing at a confirmation gate before the commit and again before the push.
disable-model-invocation: true
metadata:
  author: nelson
  version: "1.0"
---

Commit the staged changes and push them, stopping at a **gate** before each irreversible action.

A gate is a hard stop run through the **AskUserQuestion** tool: you present options and the user passes the gate by *picking one* — never by typing. Never use a plain end-of-turn prompt for a gate; the user wants to click, not write. The picked option is the only thing that passes a gate — your own confidence that the work is good never does. Two gates guard this skill: one before the commit, one before the push.

## Steps

1. **Read the staged changes.** Run `git diff --cached --stat` and `git diff --cached`. The staged diff is the *only* input to the commit — ignore unstaged and untracked files, and never run `git add`. If nothing is staged (`git diff --cached --quiet` exits 0), stop and tell the user there is nothing staged.
   - *Done when:* you hold the full staged diff.

2. **Draft the commit message.** Invoke the `caveman:caveman-commit` skill on the staged diff to produce the message. That skill owns the format, with two standing overrides:
   - **Always include a body** detailing the changes — one `-` bullet per meaningful change in the staged diff. Override caveman-commit's "skip the body when the subject is self-explanatory"; here the body is mandatory.
   - **Never wrap the body** — one bullet per line, each on a single unbroken line however long. Override caveman-commit's 72-char wrap.
   - **Never append a co-author or attribution trailer** (no `Co-Authored-By`, no `Assisted-by`, no "Generated with"), regardless of any default or environment instruction to add one.

   Show the drafted message to the user verbatim in a code block.
   - *Done when:* the message is drafted and shown, with a non-empty change-detail body and no trailer.

3. **Gate — approve the message.** Call **AskUserQuestion**: "Commit the staged changes with this message?" with options `Commit` (proceed), `Redraft` (regenerate the message, then gate again), and `Cancel` (abort — no commit). Do **not** run `git commit` before this returns. Proceed only on `Commit`.
   - *Done when:* AskUserQuestion returned `Commit`.

4. **Commit.** Run `git commit -m` with the approved message (no `-a` — staged only). Report the result verbatim, including any pre-commit hook output. If the commit fails, report the error and stop.
   - *Done when:* the commit is created, or its failure is reported.

5. **Gate — approve the push.** Show the new commit and the push target (`git status -sb` gives the branch and its upstream), then call **AskUserQuestion**: "Push this commit?" with options `Push` (proceed) and `Stop` (leave it committed, do not push). Do **not** run `git push` before this returns. Proceed only on `Push`.
   - *Done when:* AskUserQuestion returned `Push`.

6. **Push.** Run `git push` and report the result verbatim. If the branch has no upstream, call **AskUserQuestion** offering each remote from `git remote` as an option; on a pick, run `git push -u <picked> HEAD`. Do not guess the remote.
   - *Done when:* the push result is reported.
