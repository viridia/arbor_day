# Agent instructions

`arbor_day` makes low-poly plants for Panoply (`../panoply`): a generator, and a Bevy app for
authoring species with it. It is a Rust port of `../seedling` (TypeScript, three.js), with a new
growth model.

## Orient

1. Read `docs/ledger.md`: what has landed, what is next, deferrals. How it works is in
   `docs/design.md` (`TD<n>`) and why in `docs/decisions.md` (`D<n>`); read those by section.
2. Check `git log --oneline -5`.
3. `docs/requirements.md` says what the tool must do.

## Rules

- Never commit, push, or stage. The author reviews and commits; draft the commit message: a
  short subject line (`feat:` / `fix:` / `refactor:` / `chore:` / `docs:` prefix, under about 72
  characters), a blank line, then a body of bullets, one per change, wrapped at 72 columns.
- One chunk at a time: one focused change. Update the docs in the same chunk: the ledger's
  Landed row, Next and deferrals; `docs/design.md` for what changed in how it works;
  `docs/decisions.md` for a new decision; `docs/guidelines.md` for a correction made or expected.
- Before scoping a chunk, read `docs/guidelines.md`.
- Every open item goes in the ledger: in Next, or in Deferred with a gate.
- Propose structural changes (new dependencies, a new crate, replacing a subsystem) with a
  recommendation before writing code.
- Bevy is main, named by branch (D1). Read APIs at the commit `Cargo.lock` holds, in
  Cargo's git checkout of it (ledger, Notes), and its `_release-content/migration-guides/`,
  rather than guessing. Prefer Bevy's built-in replacements over third-party crates.
- `../seedling` is a reference for what the old tool did, not how to do it; the requirements say
  where it went wrong.
- Patterns here are meant for Panoply's editor (D3). When one settles, say so in the
  ledger, so Panoply can take it.
- Prose follows the "House style" section of `../bevy_action_map/CLAUDE.md`.

## Verify

- `cargo clippy --all-targets`: no errors, no new warnings. `cargo test`.
- Runtime: shader and pipeline errors only appear at runtime. Run a plan with the remote driver,
  `python3 ../bevy_remote_driver/client/run.py plans/smoke.py`, which builds and launches the
  app, runs the steps, saves screenshots under `target/remote-driver/`, and quits. Check the
  behaviour a chunk adds in the existing plan it fits, following
  `../bevy_remote_driver/docs/writing-plans.md`: each plan is a launch whose window takes focus,
  so a new plan only for a new area. `plans/gallery/` holds screenshot galleries with no checks,
  which `run.py plans` passes over; run one by path when the generator's output changes.
