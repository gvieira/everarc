# AGENTS.md

See [README.md](./README.md) for the project overview.

## Collaboration style

Keep plans and implementation narrowly scoped into small, reviewable
architectural chunks. After each chunk, suggest a practical file-review
order and wait for Gui's explicit approval before starting the next chunk.

Before starting a chunk of something meaninfully big, write a plan
to `tmp/<goal-name>.md` describing scope and approach. Proposing a chunk
and asking a scoping question is not approval—do not write, edit, or run
anything for that chunk until Gui explicitly approves the plan file.

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

**Cats (10 → 10):** Mochi, Waffles, Biscuit, Nimbus, Pixel, Garfunkel, Soot,
Tofu, Captain, Olive

No entries yet.
