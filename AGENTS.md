# AGENTS.md

**Start here**: Read [Cargo.toml](Cargo.toml) for understanding the project structure.

## Project Overview

This is a Ratatui based Terminal application much like a tui for the [Music Player daemon](https://www.musicpd.org/).

## Crates

This workspace includes a tui application acting as a client for the mpd, and also has a MPD (client)[./crumzi-client] API
trying to implement functionaliry like the [mpd](https://docs.rs/mpd/latest/mpd/) crate.

## Building

```bash
cargo build                 # Build all crates
cargo build -p crumzi-cli   # Build only cli crate 
```

## Testing

Not all tests are implemented yet. but still

```bash
cargo test
```

## Formatting and Linting

Always run these before committing:

```bash
cargo fmt
cargo clippy
```

