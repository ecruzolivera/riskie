---
status: in-progress
phase: 4
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

## Phase 4: Error Handling & Polish [PENDING]

- [ ] 4.1 Handle mount failures gracefully
- [ ] 4.2 Handle unmount failures (device busy)
- [ ] 4.3 Add optional notifications (notify-rust)
- [ ] 4.4 Configuration file support (optional)
- [ ] 4.5 Systemd service unit file

## Phase 5: Testing & Documentation [PENDING]

- [ ] 5.1 Manual testing on i3/Sway/Hyprland
- [ ] 5.2 Write README with installation instructions
- [ ] 5.3 Add example config file
- [ ] 5.4 Document systemd integration

## Technical Architecture

```
riskie daemon
├── D-Bus client (zbus)
│   ├── Connect to udisks2
│   ├── Listen for InterfacesAdded/Removed signals
│   └── Query Block/Filesystem interfaces
├── Device Manager
│   ├── Track devices in HashMap<DevicePath, DeviceInfo>
│   ├── Automount on device addition
│   └── Cleanup on device removal
├── System Tray (ksni)
│   ├── Show icon in system tray
│   ├── Menu: List devices with mount/unmount actions
│   └── Update menu on device changes
└── Mount Point Manager
    ├── Create /run/media/$USER/{label}
    ├── Call udisks2 Mount() method
    └── Call udisks2 Unmount() method
```

## Key Questions (Need Research)

1. **zbus API for udisks2:** Need to understand the exact D-Bus interfaces and methods (zbus proxy generation)
2. **ksni dynamic menu:** How to update menu items when devices change?
3. **Mount point naming:** How to handle label conflicts (duplicate labels)?
4. **Permissions:** Does riskie need to be in specific groups? (storage, plugdev)
5. **udisks2 API:** Need to study `org.freedesktop.UDisks2.*` interfaces

## Dependencies Research Needed

- [ ] D-Bus interface documentation for udisks2
- [ ] zbus proxy code generation examples
- [ ] ksni menu update patterns
- [ ] How udisks2 determines mount point names
- [ ] Best practices for daemon lifecycle in Rust

## Notes

- 2026-03-23: Initial plan creation. Scope focuses on daemon mode + system tray only, prioritizing simple UX over udiskie's cascading menus.
- 2026-03-23: Target minimal window managers (i3, Hyprland, Sway) where built-in automounting is absent.
- 2026-03-23: Chose ksni over tray-icon for system tray. Reasons: D-Bus native (no gtk dep), works on X11+Wayland, consistent with udisks2 D-Bus architecture.
