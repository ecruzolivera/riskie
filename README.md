# riskie

A simple, opinionated disk automounting daemon for Linux written in Rust.

## Overview

riskie is a Rust implementation of udiskie, designed to be simpler and more opinionated. It automatically mounts removable devices and provides a system tray interface for easy management.

### Features

- ✅ Automount removable devices to `/run/media/$USER`
- ✅ System tray interface (ksni) - works on X11 and Wayland
- ✅ D-Bus integration with udisks2
- ✅ Desktop notifications on mount/unmount events
- ✅ Mount/unmount from tray menu (left or right click)
- ✅ Target: i3, Hyprland, Sway (minimal window managers)

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
```

The binary will be at `target/release/riskie`.

### Pre-built Binaries

(Coming soon)

### System-wide Installation

```bash
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

### Running Directly

```bash
# Run the daemon
riskie

# With verbose logging
RUST_LOG=info riskie
```

### Systemd Service (Recommended)

1. Copy the service file:

```bash
mkdir -p ~/.config/systemd/user/
cp contrib/riskie.service ~/.config/systemd/user/
```

2. Enable and start:

```bash
systemctl --user enable --now riskie
```

3. Check status:

```bash
systemctl --user status riskie
```

4. View logs:

```bash
journalctl --user -u riskie -f
```

## System Tray Usage

- **Left or Right click** on the tray icon to open the menu
- Click on a device to **mount** (if unmounted) or **unmount** (if mounted)
- Click **Exit** to quit the daemon

## Desktop Notifications

riskie sends desktop notifications for:
- Device connected
- Mount success/failure
- Unmount success/failure

If unmount fails because the device is busy, the notification will suggest closing open files.

## Configuration

Currently, riskie is opinionated and does not support configuration files. All behavior is hardcoded:

- **Mount points**: `/run/media/$USER/{device_label}` (handled by udisks2)
- **Auto-mount**: Enabled by default for all removable devices
- **Notifications**: Enabled by default

## Development Status

**Phase 1: Core D-Bus Integration** - ✅ COMPLETE
**Phase 2: Device Management** - ✅ COMPLETE
**Phase 3: System Tray** - ✅ COMPLETE
**Phase 4: Error Handling & Polish** - ✅ COMPLETE
**Phase 5: Testing & Documentation** - 🔄 IN PROGRESS

## Architecture

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
│   └── Update menu dynamically
├── Notifications (notify-rust)
│   ├── Device connected
│   ├── Mount success/failure
│   └── Unmount success/failure
└── Mount Point Manager
    ├── Call udisks2 Mount() method
    └── Call udisks2 Unmount() method
```

## Dependencies

- `zbus` - D-Bus bindings for Rust
- `ksni` - StatusNotifierItem implementation (system tray)
- `notify-rust` - Desktop notifications
- `tokio` - Async runtime
- `tracing` - Logging and tracing
- `anyhow` - Error handling

## License

MIT or Apache-2.0

## Acknowledgments

Inspired by [udiskie](https://github.com/coldfix/udiskie) by coldfix.