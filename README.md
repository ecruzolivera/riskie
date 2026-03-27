# riskie

![Screenshot](docs/screenshot.png)

A simple, opinionated disk automounting daemon for Linux written in Rust.

## Overview

riskie is a Rust implementation of udiskie, designed to be simpler and more opinionated. It automatically mounts removable devices and provides a system tray interface for easy management.

### Features

- ✅ Automount removable devices to `/run/media/$USER`
- ✅ System tray interface (ksni) - works on X11 and Wayland
- ✅ D-Bus integration with udisks2
- ✅ Desktop notifications on mount/unmount events
- ✅ Mount/unmount/eject from tray menu
- ✅ LUKS encrypted device support (unlock/lock/eject)
- ✅ Multi-language support (English, Spanish)
- ✅ Target: i3, Hyprland, Sway (minimal window managers)

### Main differences from udiskie

- **Simpler UX**: Single-click mount/unmount from tray (no cascading menus)
- **Daemon-only mode**: No one-shot mode, designed to run as a background service

## Installation

### Arch Linux (AUR)

```bash
# Using yay
yay -S riskie

# Using paru
paru -S riskie

# Enable autostart
systemctl --user enable --now riskie.service
```

### From Source

```bash
git clone https://github.com/ecruzolivera/riskie.git
cd riskie
cargo build --release
```

The binary will be at `target/release/riskie`.

**Build Requirements:**

- Rust toolchain (cargo, rustc)
- gettext (for compiling translations)

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/ecruzolivera/riskie/releases).

### System-wide Installation

```bash
sudo install -m 755 target/release/riskie /usr/local/bin/

# Install translations
for po in po/*.po; do
    lang=$(basename "$po" .po)
    install -Dm644 "$po" "/usr/share/locale/$lang/LC_MESSAGES/riskie.mo"
done

# Install systemd service
install -Dm644 contrib/systemd/riskie.service /usr/lib/systemd/user/
```

### Dependencies

**Runtime:**

- **udisks2**: Must be running (standard on most Linux distributions)
- **D-Bus**: Required for communication with udisks2
- **gettext**: For loading translations at runtime
- **System tray**: One of:
  - i3bar/Swaybar with tray support
  - Waybar
  - eww
  - Any StatusNotifierItem-compatible tray

### Supported Languages

- English (default)
- Spanish (es)

### Adding a Translation

1. Copy the template:

   ```bash
   cp po/riskie.pot po/{lang}.po
   ```

2. Edit the `.po` file with your translations

3. Build and install:
   ```bash
   msgfmt po/{lang}.po -o {lang}.mo
   sudo install -Dm644 {lang}.mo /usr/share/locale/{lang}/LC_MESSAGES/riskie.mo
   ```

## Configuration

Currently, riskie is opinionated and does not support configuration files. All behavior is hardcoded:

- **Mount points**: `/run/media/$USER/{device_label}` (handled by udisks2)
- **Auto-mount**: Enabled by default for all removable devices
- **Notifications**:Enabled by default

## Dependencies

- `zbus` - D-Bus bindings for Rust
- `ksni` - StatusNotifierItem implementation (system tray)
- `notify-rust` - Desktop notifications
- `gettext-rs` - Internationalization
- `tokio` - Async runtime
- `tracing` - Logging and tracing
- `anyhow` - Error handling

## Troubleshooting

### Collecting Logs for Bug Reports

When reporting issues, please include relevant logs to help diagnose the problem.

#### Enable Verbose Logging

```bash
# Run with verbose logging
RUST_LOG=info riskie

# For more detailed debug output
RUST_LOG=debug riskie
```

#### Collect Logs from Systemd

If running as a systemd user service:

```bash
# View recent logs
journalctl --user -u riskie -n 100

# Follow logs in real-time
journalctl --user -u riskie -f

# Save logs to file
journalctl --user -u riskie --since "1 hour ago" > riskie-logs.txt
```

## License

MIT License - see [LICENSE](LICENSE)

## Acknowledgments

Inspired by [udiskie](https://github.com/coldfix/udiskie) by coldfix.
