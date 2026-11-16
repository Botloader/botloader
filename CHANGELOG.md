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

### Added

- `flags` on all channels (`ChannelFlags`): `pinned`, `requireTag`,
  `hideMediaDownloadOptions`, `channelObfuscated` and `isSpoilerChannel`.
- `SystemChannelFlags.suppressRoleSubscriptionPurchaseNotifications` and
  `SystemChannelFlags.suppressRoleSubscriptionPurchaseNotificationReplies`.

## [2026.1.1] - 2026-11-15

### Added

- `MessageFlags.hasSnapshot` for forwarded messages.
- Numeric channel and message types: channels, channel mentions, invite
  channels, interaction channels, thread delete events, messages, message
  snapshots and message update events now have a `kindId` field with the raw
  discord type id, alongside the existing `kind` name. Use the new
  `ChannelTypes` and `MessageTypes` constants to compare against them.
- Everywhere a channel type is accepted (settings channel options, command
  channel options, channel select menus, `createChannel`) a raw discord channel
  type id can now be passed instead of a name, e.g `ChannelTypes.Forum` or `20`.
- Channel and message types botloader doesn't know yet no longer cause errors
  (e.g "Unimplemented channel type 20" from `getChannels()`). They get
  `kind: "Unknown"` and channels become an `UnknownChannel`, use `kindId` to
  tell them apart.

### Changed

- Updated twilight to 0.18.
- Updated deno_core to 0.413 (V8 15.0).
- `Webhook.channelId` is now `string | null`, since Discord can return webhooks
  without a channel.

### Fixed

- Scripts could abort the vm worker process they run in by calling the internal `op_panic` op
  through `Deno.core.ops`. This and other internal deno_core ops scripts have no use for are now
  disabled.
- `MessageFlags.isComponentsV2` on received messages reflected the
  `suppressNotifications` flag instead of the components v2 flag.
- Media channels are now `TextChannel`s in scripts (they were `UnknownChannel`),
  and can be used as a channel type filter in settings channel pickers.

## [2026.1.0] - 2026-07-05

### Added

- Release versioning: `YEAR.MAJOR.MINOR` version embedded in the backend binary,
  reported via `backend --version`, the `botloader_build_info` metric, and the
  Sentry release field. CI publishes version-tagged Docker images on release tags.
- `scripts/release.sh` for cutting releases: bumps the `VERSION` file, rolls the
  changelog, and creates the release commit and tag.
