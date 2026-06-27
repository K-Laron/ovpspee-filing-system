# Plan 010: Rust unit tests for auth module

> **Executor instructions**: Follow step by step. When done, update `plans/README.md`.
>
> **Drift check**: `git diff --stat 9b4b638..HEAD -- src-tauri/src/auth.rs src-tauri/src/documents.rs`

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: 003 (auth guards consolidation — avoids writing tests against code that's about to change)
- **Category**: tests
- **Planned at**: commit `9b4b638`, 2026-06-27

## Why this matters

The auth module (310 lines) has zero `#[cfg(test)]` unit tests. Password validation, username validation, session expiry logic, and first-run detection have no sub-second feedback loop. A developer must run the full integration test suite (5+ seconds with DB setup) to catch regressions in auth logic. Adding in-module unit tests gives instant (sub-second) feedback for the most security-critical module.

## Current state

- `src-tauri/src/auth.rs:1-310` — zero `#[cfg(test)]` blocks.
- Integration tests exist in `src-tauri/tests/auth_slice1.rs` that cover some auth flows (login, session validation, deactivated user) but these require DB setup and migration.
- `create_test_pool()` exists in `src-tauri/src/db.rs:32-41` for in-memory SQLite test pools.
- Pure functions that can be unit-tested without DB:
  - `validate_password(password: &str) -> AppResult<()>` (line 33)
  - `validate_username(username: &str) -> AppResult<()>` (line 52)
  - `hash_password(password: &str) -> AppResult<String>` (line 70) — calls `validate_password` internally, then hashes
  - `verify_password(password: &str, password_hash: &str) -> AppResult<()>` (line 82)

**Repo conventions to follow**: Test module at the bottom of the file with `#[cfg(test)]`. Use `super::*` to import module functions. Follow the existing pattern from `src-tauri/src/printing.rs:200+` for minimal-test structure.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password_rejects_short() {
        let result = validate_password("Ab1!");
        assert!(result.is_err());
    }
}
```

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Rust test | `cargo test --manifest-path src-tauri/Cargo.toml` | all pass |

## Scope

**In scope**:
- `src-tauri/src/auth.rs` — add `#[cfg(test)] mod tests { ... }` at end of file

**Out of scope**:
- Other Rust modules (documents, backup, etc.) — add tests only to auth.rs in this plan
- Integration tests (they already exist)
- Any production code changes

## Steps

### Step 1: Add unit test module to auth.rs

Add at the end of `src-tauri/src/auth.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password_min_length() {
        assert!(validate_password("Ab1!").is_err());
        assert!(validate_password("Ab1!defg").is_ok());
    }

    #[test]
    fn test_validate_password_requires_digit() {
        assert!(validate_password("Abcdefgh!").is_err());
        assert!(validate_password("Abcdefg1!").is_ok());
    }

    #[test]
    fn test_validate_password_requires_special() {
        assert!(validate_password("Abcdefg1").is_err());
        assert!(validate_password("Abcdefg1!").is_ok());
    }

    #[test]
    fn test_validate_username_min_length() {
        assert!(validate_username("ab").is_err());
        assert!(validate_username("abc").is_ok());
        assert!(validate_username("a").is_err());
    }

    #[test]
    fn test_validate_username_max_length() {
        let long = "a".repeat(51);
        assert!(validate_username(&long).is_err());
        let ok = "a".repeat(50);
        assert!(validate_username(&ok).is_ok());
    }

    #[test]
    fn test_validate_username_valid_chars() {
        assert!(validate_username("user_name").is_ok());
        assert!(validate_username("user-name").is_ok());
        assert!(validate_username("user name").is_err());
        assert!(validate_username("user!name").is_err());
    }

    #[test]
    fn test_hash_password_returns_hash() {
        let hash = hash_password("ValidP@ss1").unwrap();
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_hash_password_rejects_weak() {
        assert!(hash_password("short").is_err());
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "ValidP@ss1";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash).is_ok());
    }

    #[test]
    fn test_verify_password_wrong() {
        let hash = hash_password("ValidP@ss1").unwrap();
        assert!(verify_password("WrongP@ss1", &hash).is_err());
    }
}
```

**Verify**: `cargo test --manifest-path src-tauri/Cargo.toml` — all tests pass, including the new unit tests. Run with `-- --nocapture` to see individual test names:

```
cargo test --manifest-path src-tauri/Cargo.toml auth::tests -- --nocapture
```

Expected output:
```
test auth::tests::test_validate_password_min_length ... ok
test auth::tests::test_validate_password_requires_digit ... ok
// ... all 10+ tests pass
```

## Test plan

- 10+ new unit tests covering:
  - Password validation: min length, requires digit, requires special character
  - Username validation: min/max length, valid characters
  - Password hashing: returns valid Argon2id hash, rejects weak passwords
  - Password verification: correct password passes, wrong password fails
- These tests run in < 100ms without any DB setup.

## Done criteria

- [ ] `cargo test --manifest-path src-tauri/Cargo.toml` — all tests pass
- [ ] `grep -n "#\[cfg(test)\]" src-tauri/src/auth.rs` — at least one match
- [ ] `grep -c "#\[test\]" src-tauri/src/auth.rs` — at least 8 test functions
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml auth::tests -- --nocounter` — shows all new tests passing
- [ ] `plans/README.md` status row updated

## STOP conditions

- If `hash_password` or `verify_password` require specific Argon2id parameters, the test should match. Check the current parameters in the code (line 73: `Params::new(65_536, 3, 4, None)`) — the test assertions should work with these defaults.
- If tests fail on CI due to timing or environment, run with `RUST_LOG=debug cargo test` to see error details.

## Maintenance notes

- When adding new validation functions to `auth.rs`, add corresponding unit tests in the same `#[cfg(test)]` module.
- When changing password hashing parameters, update the `test_hash_password_returns_hash` assertion if the prefix format changes.
- The DB-dependent functions (`first_run_required`, `authenticate_user`, `validate_session`) are covered by existing integration tests — don't add unit tests for them here.
