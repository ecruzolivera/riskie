---
status: complete
phase: 5
updated: 2026-03-23
---

# Implementation Plan: riskie

## Goal

Create a Rust-based disk automounting daemon with system tray interface that automatically mounts removable devices to `/run/media/$USER` with a simple click-to-unmount UX.

## Context & Decisions

| Decision                  | Rationale                                          | Source                            |
| ------------------------- | -------------------------------------------------- | --------------------------------- |
| Use zbus for D-Bus        | Mature Rust D-Bus library with async support       | Community recommendation          |
| Use ksni for system tray  | Native D-Bus StatusNotifierItem, no gtk dep, works on X11+Wayland | Research: tray library comparison |
| Automount all removable   | Opinionated: simpler UX, mount everything          | User requirement                  |
| Mount to /run/media/$USER | FHS-compliant, avoids permission issues            | User requirement                  |
| Daemon-only mode          | Simplify initial scope, no one-shot mode           | User requirement                  |
| System tray-first UX      | Direct menu for mount/unmount (no cascading menus) | User requirement                  |
| Target i3/Hyprland/Sway   | Minimal environments lack built-in automounting    | User requirement                  |
| notify-rust for notifications | D-Bus native desktop notifications             | Standard Linux desktop integration |

## Phase 1: Core D-Bus Integration [COMPLETE]

- [x] 1.1 Setup project structure and Cargo.toml
- [x] 1.2 Basic D-Bus connection to udisks2
- [x] 1.3 Subscribe to device added/removed signals
- [x] 1.4 Enumerate existing devices on startup
- [x] 1.5 Basic logging setup with tracing

## Phase 2: Device Management [COMPLETE]

- [x] 2.1 Track mounted/unmounted devices in memory
- [x] 2.2 Implement automount logic (removable devices only)
- [x] 2.3 Implement unmount logic
- [x] 2.4 Handle device removal (cleanup mounts)
- [x] 2.5 Mount to /run/media/$USER/{label-or-uuid} (handled by udisks2)

## Phase 3: System Tray [COMPLETE]

- [x] 3.1 Create system tray icon (ksni)
- [x] 3.2 Build device menu (show removable devices)
- [x] 3.3 Add mount/unmount actions to menu items
- [x] 3.4 Update menu dynamically when devices change
- [x] 3.5 Show mount status indicators in menu
- [x] 3.6 Left-click and right-click open menu (MENU_ON_ACTIVATE)

## Phase 4: Error Handling & Polish [COMPLETE]

- [x] 4.1 Handle mount failures gracefully with notifications
- [x] 4.2 Handle unmount failures (device busy) with user feedback
- [x] 4.3 Add desktop notifications (notify-rust)
- [x] 4.4 Fix RwLock blocking in async context
- [x] 4.5 Fix mount point parsing (byte arrays vs ObjectPath)
- [x] 4.6 Create systemd service unit file

## Phase 5: Testing & Documentation [COMPLETE]

- [x] 5.1 Write README with installation instructions
- [x] 5.2 Document systemd integration
- [x] 5.3 Create contrib/riskie.service template

## Technical Architecture

```
riskie daemon
├── D-Bus client (zbus)
│   ├── Connect to udisks2
│   ├── Listen for InterfacesAdded/Removed signals
│   └── Query Block/Filesystem interfaces
├── Device Manager
│   ├── Track devices in Vec<Device>
│   ├── Automount on device addition
│   └── Cleanup on device removal
├── System Tray (ksni)
│   ├── Show icon in system tray
│   ├── Menu: List devices with mount/unmount actions
│   └── Update menu dynamically (uses RwLock for state)
├── Notifications (notify-rust)
│   ├── Device connected notification
│   ├── Mount success/failure notifications
│   └── Unmount success/failure notifications
└── Mount Point Manager
    ├── Call udisks2 Mount() method
    └── Call udisks2 Unmount() method
```

## Files

```
src/
├── main.rs        - Entry point, event loop, device tracking
├── tray.rs        - System tray implementation (ksni)
├── udisks2.rs     - D-Bus udisks2 client
└── notify.rs      - Desktop notifications

contrib/
└── riskie.service - Systemd user service template
```

## Dependencies

| Crate          | Purpose                        |
| -------------- | ------------------------------ |
| tokio          | Async runtime                  |
| zbus           | D-Bus bindings                 |
| ksni           | StatusNotifierItem (tray)      |
| notify-rust    | Desktop notifications          |
| tracing        | Logging                        |
| tracing-subscriber | Logging initialization     |
| anyhow         | Error handling                 |
| thiserror      | Error types                    |
| futures        | Async utilities                |
| async-stream   | Stream macros                  |

## Notes

- 2026-03-23: Initial plan creation. Scope focuses on daemon mode + system tray only, prioritizing simple UX over udiskie's cascading menus.
- 2026-03-23: Target minimal window managers (i3, Hyprland, Sway) where built-in automounting is absent.
- 2026-03-23: Chose ksni over tray-icon for system tray. Reasons: D-Bus native (no gtk dep), works on X11+Wayland, consistent with udisks2 D-Bus architecture.
- 2026-03-23: Fixed critical issues: RwLock blocking in async context, mount point byte array parsing, try_send for tray commands.
- 2026-03-23: Added desktop notifications for all mount/unmount events.