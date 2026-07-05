# Changelog

All notable changes to Botloader are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
using a `YEAR.MAJOR.MINOR` version scheme (e.g. `2026.5.1`):

- **YEAR**: the calendar year of the release
- **MAJOR**: incremented for significant releases within the year (resets each year)
- **MINOR**: incremented for smaller fixes/patches on top of a major release

## Releasing

1. Run the release script from a clean master checkout:

   ```sh
   scripts/release.sh minor   # or: major, or an explicit version like 2026.5.1
   ```

   It bumps the `VERSION` file, moves the `[Unreleased]` changelog entries under
   the new version, and creates the release commit and tag.

2. Push the release commit and tag with the command the script prints:

   ```sh
   git push origin master v2026.5.1
   ```

3. CircleCI builds the images with the version embedded in the binary
   (`common::VERSION`, also shown by `backend --version`) and pushes them to
   Docker Hub tagged with the version (e.g. `botloader/backend:2026.5.1`).

## [Unreleased]

## [2026.1.0] - 2026-07-05

### Added

- Release versioning: `YEAR.MAJOR.MINOR` version embedded in the backend binary,
  reported via `backend --version`, the `botloader_build_info` metric, and the
  Sentry release field. CI publishes version-tagged Docker images on release tags.
- `scripts/release.sh` for cutting releases: bumps the `VERSION` file, rolls the
  changelog, and creates the release commit and tag.
