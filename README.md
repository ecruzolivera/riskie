# riskie

A simple, opinionated disk automounting daemon for Linux written in Rust.

## Overview

riskie is a Rust implementation of udiskie, designed to be simpler and more opinionated. It automatically mounts removable devices and provides a system tray interface for easy management.

### Features

- ✅ Automount removable devices to `/run/media/$USER`
- ✅ System tray interface (ksni)
- ✅ D-Bus integration with udisks2
- ✅ Target: i3, Hyprland, Sway (minimal environments)

### Difference from udiskie

- **Simpler UX**: Single-click mount/unmount from tray (no cascading menus)
- **Opinionated defaults**:
  - Automount ALL removable devices
  - Mount to `/run/media/$USER/{label}` (FHS-compliant)
  - No LUKS support (keeps it simple)
- **Daemon-only mode**: No one-shot mode, designed to run as a background service
- **Modern Rust**: Better performance, smaller binary, async-first design

## Installation

### From Source

```bash
git clone https://github.com/yourusername/riskie.git
cd riskie
cargo build --release
sudo install -m 755 target/release/riskie /usr/local/bin/
```

### Dependencies

- **udisks2**: Must be running (standard on most Linux distributions)
- **D-Bus**: Required for communication with udisks2
- **System tray**: One of:
  - i3bar/Swaybar with tray support
  - Waybar
  - eww
  - Any StatusNotifierItem-compatible tray

## Usage

```bash
# Run the daemon
riskie

# Or with logging
RUST_LOG=riskie=info riskie
```

## Systemd Service (Optional)

Create `~/.config/systemd/user/riskie.service`:

```ini
[Unit]
Description=Riskie Disk Automounter
After=udisks2.service

[Service]
Type=simple
ExecStart=/usr/local/bin/riskie
Restart=on-failure

[Install]
WantedBy=default.target
```

Enable and start:

```bash
systemctl --user enable --now riskie
```

## Configuration

Currently, riskie is opinionated and does not support configuration files. All behavior is hardcoded:

- **Mount points**: `/run/media/$USER/{device_label}`
- **Auto-mount**: Enabled by default for all removable devices
- **Notifications**: planned for future release

## Development Status

**Phase 1: Core D-Bus Integration** - ✅ COMPLETE
- [x] Project structure and Cargo.toml
- [x] D-Bus connection to udisks2
- [x] Device enumeration on startup
- [x] Device added/removed event monitoring
- [x] Basic logging with tracing

**Phase 2: Device Management** - 🔄 IN PROGRESS
- [ ] Track mounted/unmounted devices
- [ ] Automount logic
- [ ] Unmount logic
- [ ] Mount point creation

**Phase 3: System Tray** - 📋 PLANNED
- [ ] System tray icon
- [ ] Device menu
- [ ] Mount/unmount actions

## Architecture

```
riskie daemon
├── D-Bus client (zbus)
│   ├── Connect to udisks2
│   ├── Listen for InterfacesAdded/Removed signals
│   └── Query Block/Filesystem interfaces
├── Device Manager
│   ├── Track devices in HashMap
│   ├── Automount on device addition
│   └── Cleanup on device removal
├── System Tray (ksni)
│   ├── Show icon in system tray
│   ├── Menu: List devices with mount/unmount actions
│   └── Update menu dynamically
└── Mount Point Manager
    ├── Create /run/media/$USER/{label}
    ├── Call udisks2 Mount() method
    └── Call udisks2 Unmount() method
```

## Dependencies

- `zbus` - D-Bus bindings for Rust
- `ksni` - StatusNotifierItem implementation (system tray)
- `tokio` - Async runtime
- `tracing` - Logging and tracing
- `anyhow` - Error handling

## License

MIT or Apache-2.0

## Acknowledgments

Inspired by [udiskie](https://github.com/coldfix/udiskie) by coldfix.