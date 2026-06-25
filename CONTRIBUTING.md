# Contributing to Bearings

Welcome — this is the working agreement so two people (and their agents) can build
on `bearings-rs` at the same time without stepping on each other.

## The one rule

**GitHub `main` is the single source of truth. The VPS is deploy-only.**

Nothing is hand-edited on the production box. Every change reaches `main` through a
branch and a pull request, and only then gets deployed.

## Day-to-day flow

1. **Branch off `main`.** Develop in a clone of the repo, or in the VPS dev worktree
   at `/opt/bearings-dev` (kept separate from the deploy checkout `/opt/bearings-rs`).
   ```bash
   git switch main && git pull
   git switch -c feat/short-description
   ```
2. **Commit small, push, open a PR.**
   ```bash
   git push -u origin HEAD
   gh pr create --fill
   ```
3. **CI gates the merge — not a human.** The `Check + Test + Lint` workflow
   (`cargo check`, `cargo test --lib`, `clippy -D warnings`, `cargo fmt --check`)
   must be green. **No review approval is required.** Let it land automatically:
   ```bash
   gh pr merge --auto --squash
   ```
4. **Deploy** when you want the change live:
   ```bash
   ssh root@<vps>          # then:
   cd /opt/bearings-rs && ./deploy.sh
   ```

## Conventions

- **Never commit to `main` directly**, and never hand-edit `/opt/bearings-rs`.
- **Keep PRs small** and focused — easier to read, less likely to collide.
- **Pull `main` before starting** a new branch.
- **Secrets** live in `.env` on the VPS only — never commit them (`git add -p`, not `git add -A`).
- Branch names: `feat/…`, `fix/…`, `chore/…`, `docs/…`.

## Repo map

See `README.md` (project overview) and `bearings-backend/ARCHITECTURE.md`
(backend design + decisions). The backend layering is
`routes/ssr → repositories → db (Supabase PostgREST)`.
