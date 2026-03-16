# Async MPD client roadmap (`crumzi-client`)

## Goal
Implement an **async** MPD client crate with an API that feels similar to the Rust `mpd` crate, using **Tokio** IO, while keeping protocol handling correct, testable, and extensible.

Initial milestone scope:
- **Core**: connection handshake, command framing, response parsing, OK/ACK handling.
- **Queue**: add/delete/clear/move and queue listing.

## Architecture overview
Design the crate as **two layers**:

- **Protocol layer (private or `pub(crate)`):**
  - Responsible for framing commands, writing to the socket, reading lines, parsing key/value pairs, detecting list boundaries, and handling `OK` vs `ACK`.
  - Produces structured intermediate representations.
- **High-level client API (public):**
  - Methods similar to `mpd::Client` but `async fn`.
  - Converts protocol responses into typed structs: `Song`, `Status`, etc.

## Module layout (implemented)
- `src/lib.rs`: `Client` public surface and re-exports.
- `src/error.rs`: unified error type (IO, protocol, parse, server ACK).
- `src/proto/`: command building + response parsing.
  - `src/proto/command.rs`
  - `src/proto/response.rs`
  - `src/proto/mod.rs`
- `src/types/`: typed models and parsers
  - `src/types/song.rs`
- `src/queue.rs`: queue-related client methods.

## Implemented public API (queue milestone)
On `Client<S>`:
- `connect(addr) -> Client<TcpStream>`
- `server_version() -> &str`
- `add(uri)`
- `add_id(uri) -> u32`
- `clear()`
- `delete(pos)`
- `move_song(from, to)`
- `playlistinfo() -> Vec<Song>`
- `playlistinfo_range(start, end) -> Vec<Song>`

## Testing strategy
- Unit tests validate:
  - command escaping/quoting
  - `ACK` parsing
  - key/value pair parsing
  - transcript-based response reading (`OK` terminator)
  - `Song` record grouping/parsing

## Next expansions (not yet implemented)
- `status`, `currentsong`, playback controls
- library browsing/search (`list`, `find`, `search`, `lsinfo`)
- idle subsystem (`idle` / `noidle`)
- binary responses (album art)

