# Repository Guidelines

## Project Structure & Module Organization
`src/main.rs` is the Axum entrypoint (routes, server bootstrap, static serving). `src/db.rs` contains SQLx models and async CRUD access.  
HTML templates live in `templates/` (Askama compile-time requirement), and styles/assets live in `static/`.  
Database schema changes go in `migrations/` with timestamped SQL files (for example, `20260523120000_init.sql`).  
Container and deployment files are at repo root: `Dockerfile`, `docker-compose.yml`, and CI in `.github/workflows/docker-image.yml`.

## Build, Test, and Development Commands
- `cargo run`: starts the app locally on `http://localhost:8080` and applies migrations at startup.
- `cargo check`: fast compile validation without producing a binary.
- `cargo test`: runs Rust tests (add tests as features evolve; currently minimal coverage).
- `cargo fmt --all`: formats code using Rustfmt conventions.
- `cargo clippy --all-targets --all-features -D warnings`: lint gate for PR-quality code.
- `docker compose up -d --build`: builds and runs the production-like stack with persisted DB in `./data/`.

## Coding Style & Naming Conventions
Use Rust 2021 idioms and keep code `rustfmt`-clean (4-space indentation, trailing commas where helpful).  
Prefer `snake_case` for functions/modules/variables, `PascalCase` for structs/enums/traits, and explicit type names for DB input/output structs (for example, `CreateItemInput`).  
Keep route handlers thin; move SQL and mapping logic into `src/db.rs`. Avoid introducing Node/Tailwind dependencies; keep UI changes in `static/styles.css` and Askama templates.

## Testing Guidelines
Place unit tests in `#[cfg(test)]` blocks near the code they validate; add integration tests under `tests/` for route/database flows.  
Name tests by behavior (for example, `creates_item_with_valid_payload`).  
Before opening a PR, run: `cargo fmt --all`, `cargo clippy --all-targets --all-features -D warnings`, and `cargo test`.

## Commit & Pull Request Guidelines
Current history uses Conventional Commit style (`feat:`, `docs:`, `ci:`); keep that format and write imperative summaries.  
PRs should include: purpose, key changes, validation steps/commands run, and screenshots or short GIFs for UI/template updates.  
If DB schema changes are included, call out the migration file and compatibility impact explicitly.
