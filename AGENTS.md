# AGENTS.md

See [README.md](./README.md) for the project overview.

## Collaboration style

Implement features in small, reviewable chunks. After each chunk, report what
changed and wait for Gui's explicit approval before starting the next chunk.

Before starting a meaningfully big or architecturally consequential feature,
write one concise feature plan to `tmp/plan.md`, overwriting it for each new
feature. Use one when the work has material design choices, spans several
coordinated behaviors, or needs its scope agreed before implementation. Keep
it short and use only these sections: `Outcome` (the completed feature),
`Steps` (the full sequence of reviewable chunks), and `Next` (follow-up ideas
outside the feature). Mark each step `[ ]` or `[Done]`; after an
approved chunk, update only those markers before asking approval for the next
one. Do not require a plan for a small, well-specified lift with an obvious
implementation. Proposing a feature and asking a
scoping question is not approval—do not write, edit, or run anything for its
first chunk until Gui explicitly approves the plan.

Gui is activelly learning Rust, which means you need to be extra-explicit
when he asks questions. Do not take such questions as requests for changing
code.

## Guide

When adding or changing user-facing CLI behavior or configuration, update
`docs/guide.md` in the same implementation chunk. `everarc guide` embeds that
file in the binary.

## Memory

Never write to the persistent memory system for this project. Durable
instructions belong in this file (or another checked-in doc), where Gui
can see and review them directly.

## Git index

Gui owns the Git index. Never run commands that change it, including `git add`,
`git restore --staged`, `git reset`, `git commit -a`, or `git commit <file>`.
When Gui tells you to commit, treat that as confirmation that the current index
is ready and commit it without asking for separate confirmation. If unstaged
changes exist, report that they will not be included, but do not stage or alter
them.

## Mistake log

Every entry records a real violation of Everarc's working rules, not a style
nitpick. Each violation costs one of Gui's 10 cats. Gui chooses which cat is
lost—no exceptions, negotiation, or repeat selections.

**Cats (10 → 9):** Mochi, Waffles, Biscuit, Nimbus, Pixel, Garfunkel, Soot,
Tofu, ~~Captain~~, Olive

- Captain — Reported the staging state without explicitly inspecting the Git index first.
