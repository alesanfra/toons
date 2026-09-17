# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [0.8.0] - 2026-09-17

### Added

- Tuples are encoded as TOON arrays, matching the `json` module.
- Integers outside the 64-bit range keep their exact value in both
  directions; previously they were silently converted to floats.
- `toons.pyi` declares `__version__`.
- `toons.__toon_spec__` reports the implemented TOON specification version
  (`"3.0"`), as recommended by spec Section 13.

### Changed

- **Breaking:** Python 3.8 is now the minimum. PyO3 0.29 no longer offers the
  `abi3-py37` target, so wheels are tagged `cp38-abi3`. Python 3.7 users stay
  on 0.7.0.
- Updated PyO3 from 0.28 to 0.29, refreshed the development and
  documentation dependencies, and bumped the GitHub Actions used by CI.
- **Breaking:** values with no TOON representation (sets, generators, custom
  objects, functions) now raise `TypeError` instead of being encoded as
  `null`, and non-string object keys raise `TypeError` instead of crashing
  the interpreter.
- Invalid option values raise `ValueError` instead of being ignored:
  `delimiter` must be `","`, `"\t"`, or `"|"`; `key_folding` must be `"off"`
  or `"safe"` (`"on"` and `"always"` remain accepted aliases); `expand_paths`
  must be `"off"`, `"safe"`, or `"always"`; the decoder `indent` hint must be
  at least 1.
- Release builds enable link-time optimization and strip symbols.

### Fixed

- Encoding a structure that contains a reference cycle raised a stack
  overflow that killed the interpreter; it now raises `ValueError`. Nesting
  deeper than 1000 containers raises `ValueError` for the same reason.
- `dumps(..., delimiter="")` panicked; a panic also escaped for non-string
  object keys.
- An array nested inside an array under an object key was written with a
  stray trailing space and a misplaced header, producing output that could
  not be decoded again.
- The decoder could not read a tabular array nested inside an expanded
  array (`- [2]{a}:`).
- `expand_paths="always"` behaved like `"safe"` and left quoted dotted keys
  unexpanded.

## [0.7.0] - 2026-05-21

### Added

- `to_json()` convenience API.

### Changed

- Binary wheels are no longer built for Linux x86, s390x, ppc64le, armv7l,
  and 32-bit Windows.

## [0.6.0] - 2026-05-15

### Added

- `ToonDecodeError` with `.line` and `.source` attributes.

### Fixed

- Quoted strings containing colons are parsed as primitives in expanded
  arrays.

## Earlier releases

See the [commit history](https://github.com/alesanfra/toons/commits/main) for
releases before 0.6.0.
