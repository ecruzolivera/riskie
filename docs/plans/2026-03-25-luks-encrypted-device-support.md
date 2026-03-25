---
status: in-progress
phase: 2
updated: 2026-03-25
---

# Implementation Plan: LUKS Encrypted Device Support

## Goal

Add support for LUKS encrypted devices with password prompting via systemd-ask-password, enabling users to unlock and mount encrypted USB drives from the system tray.

## Context & Decisions

| Decision | Rationale | Source |
| -------- | --------- | ------ |
| Use systemd-ask-password for password prompting | Cross-desktop compatibility, works with GNOME/KDE/sway agents, uses kernel keyring caching | Discussion with user |
| No auto-unlock for encrypted devices | User must click "Unlock" in tray, prevents unwanted password prompts | User preference |
| Password caching for 2.5 min | Uses systemd's --keyname flag, reduces repeated password entry | systemd-ask-password docs |
| Show unlocked partitions as children of parent drive | Consistent with existing multipartition drive UI | Existing tray behavior |
| Detect encrypted via IdUsage="crypto" | Standard udisks2 property for encrypted containers | udisks2 API |
| Terminal prompt fallback not viable | riskie runs as daemon/tray app without TTY | Analysis of systemd-ask-password |

## Phase 1: Core Detection [COMPLETE]

- [x] 1.1 Add DeviceType enum to Device struct
  - Create enum: `Filesystem`, `Encrypted`, `Cleartext`, `Other`
  - Add fields: `device_type`, `cleartext_device`, `crypto_backing_device`
  - File: `src/udisks2.rs`

- [x] 1.2 Modify enumerate_devices() to detect encrypted devices
  - Check for `Encrypted` interface presence
  - Read `CleartextDevice` and `CryptoBackingDevice` properties
  - File: `src/udisks2.rs`

- [x] 1.3 Add helper methods to Device
  - `is_encrypted(&self) -> bool`
  - `is_unlocked(&self) -> bool`
  - `is_cleartext(&self) -> bool`
  - File: `src/udisks2.rs`

- [x] 1.4 Add is_encrypted filter for removable devices
  - Device detection now includes encrypted and cleartext types
  - File: `src/udisks2.rs`

## Phase 2: Encrypted Interface [COMPLETE]

- [x] 2.1 Create src/encrypted.rs module
  - Add D-Bus proxy for `org.freedesktop.UDisks2.Encrypted`
  - File: `src/encrypted.rs`

- [x] 2.2 Implement unlock_device() function
  - Call `Unlock(passphrase, options)` D-Bus method
  - Return cleartext device object path
  - File: `src/encrypted.rs`

- [x] 2.3 Implement lock_device() function
  - Call `Lock(options)` D-Bus method
  - File: `src/encrypted.rs`

- [x] 2.4 Add Client methods for encrypted devices
  - Standalone functions in encrypted.rs module
  - Files: `src/encrypted.rs`, `src/main.rs`

- [ ] 2.5 Test manual unlock via D-Bus
  - Verify Unlock returns cleartext device path
  - Verify lock/unlock cycle works

## Phase 3: Password Prompting [COMPLETE]

- [x] 3.1 Create src/password.rs module
  - File: `src/password.rs`

- [x] 3.2 Implement prompt_password() function
  - Use systemd-ask-password with --icon, --keyname, --accept-cached, --timeout
  - Return `Option<String>` (None if cancelled)
  - File: `src/password.rs`

- [x] 3.3 Add error handling for missing systemd-ask-password
  - Check if command exists
  - Log warning if unavailable
  - File: `src/password.rs`

- [x] 3.4 Add translation strings for password prompts
  - `"Enter passphrase for {}"`
  - Files: `po/riskie.pot`, `po/en.po`, `po/es.po`

## Phase 4: Event Loop Integration [PENDING]

- [ ] 4.1 Add new TrayCommand variants
  - `Unlock(String)` - object path of encrypted device
  - `Lock(String)` - object path of cleartext device
  - File: `src/tray.rs`

- [ ] 4.2 Handle DeviceType::Encrypted in device_added handler
  - Show notification "Encrypted device detected"
  - Add to tray with "Unlock" menu item
  - Do NOT auto-unlock
  - File: `src/main.rs`

- [ ] 4.3 Handle DeviceType::Cleartext in device_added handler
  - Treat similar to Filesystem
  - Link to parent encrypted device
  - File: `src/main.rs`

- [ ] 4.4 Implement Unlock command handler
  - Call prompt_password()
  - If password: call unlock_device()
  - On unlock: re-enumerate devices, find cleartext, mount
  - File: `src/main.rs`

- [ ] 4.5 Implement Lock command handler
  - Unmount cleartext partition if mounted
  - Call lock_device()
  - Refresh tray
  - File: `src/main.rs`

- [ ] 4.6 Add new notification functions
  - `notify_encrypted_device(label)` - "Encrypted device detected"
  - `notify_unlock_success(label)` - "Device unlocked"
  - `notify_unlock_error(label, error)` - "Failed to unlock"
  - File: `src/notify.rs`

## Phase 5: Tray Menu Changes [PENDING]

- [ ] 5.1 Update menu generation for DeviceType::Encrypted
  - Show drive header with "(Encrypted)" label
  - Show "Unlock {device_name}" menu item
  - Show "Eject" menu item
  - File: `src/tray.rs`

- [ ] 5.2 Update menu generation for DeviceType::Cleartext
  - Show under parent encrypted device
  - Standard mount/unmount options
  - File: `src/tray.rs`

- [ ] 5.3 Add icon differentiation for encrypted devices
  - Use "drive-harddisk-encrypted" or similar icon
  - Fall back to "drive-removable-media" if unavailable
  - File: `src/tray.rs`

- [ ] 5.4 Add "Lock & Eject" menu item for unlocked encrypted devices
  - Unmounts all partitions
  - Locks the encrypted container
  - File: `src/tray.rs`

## Phase 6: Internationalization [PENDING]

- [ ] 6.1 Add all new strings to translation files
  - `"Enter passphrase for {}"`
  - `"Encrypted device detected"`
  - `"{} connected (encrypted)"`
  - `"Unlock {}"`
  - `"Lock {}"`
  - `"Device unlocked"`
  - `"{} unlocked successfully"`
  - `"Unlock failed"`
  - `"Failed to unlock {}: {}"`
  - `"Password prompt cancelled"`
  - `"Lock & Eject"`
  - Files: `po/riskie.pot`, `po/en.po`, `po/es.po`

- [ ] 6.2 Update build.rs if needed
  - Verify new strings compile correctly
  - File: `build.rs`

## Phase 7: Testing & Documentation [PENDING]

- [ ] 7.1 Manual testing with LUKS USB drive
  - Insert encrypted USB, verify detection
  - Click "Unlock", enter password, verify mount
  - Eject, verify lock/unmount
  - Test password caching (insert again within 2.5 min)

- [ ] 7.2 Test error scenarios
  - Wrong password entry
  - Password prompt cancelled
  - Device removed while locked/unlocked
  - systemd-ask-password not available

- [ ] 7.3 Update README.md
  - Add LUKS support to features list
  - Remove "No LUKS support" from differences with udiskie
  - Document systemd-ask-password requirement

- [ ] 7.4 Update AGENTS.md
  - Add DeviceType enum documentation
  - Add encrypted device handling notes

## Phase 8: Release [PENDING]

- [ ] 8.1 Update Cargo.toml version
  - Bump to 0.3.0 (minor version for new feature)

- [ ] 8.2 Update CI workflow if needed
  - Verify build still passes
  - File: `.github/workflows/build.yml`

- [ ] 8.3 Create release tag
  - Tag: v0.3.0

## Notes

- 2026-03-25: User confirmed design decisions: click to unlock (no auto), password caching via systemd, parent drive shows unlocked partitions
- 2026-03-25: systemd-ask-password uses kernel keyring for caching (~2.5 min timeout)
- 2026-03-25: Desktop environments provide password agents (GNOME, KDE) or require setup (i3, sway)
- udisks2 Encrypted interface: `Unlock(passphrase, options) -> cleartext_device_path`
- Cleartext device has `CryptoBackingDevice` property pointing back to encrypted parent
- Device detection: `IdUsage=="crypto"` && `IdType=="crypto_LUKS"` indicates LUKS