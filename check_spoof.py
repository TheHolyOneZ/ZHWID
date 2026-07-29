#!/usr/bin/env python3
"""
Z-HWID spoof checker — shows your current hardware/network identity and flags
what's currently spoofed.

MAC spoofing is detected reliably by comparing the live MAC against the hardware
permanent MAC (ethtool -P). machine-id / hostname are compared against Z-HWID's
backup at ~/.local/share/z-hwid/state.json when it exists.

Pure stdlib, no dependencies:  python3 check_spoof.py
"""
import json
import os
import subprocess
import sys
from pathlib import Path

USE_COLOR = sys.stdout.isatty()


def c(text, color):
    if not USE_COLOR:
        return text
    codes = {"green": 32, "red": 31, "yellow": 33, "dim": 90, "bold": 1, "cyan": 36}
    return f"\033[{codes[color]}m{text}\033[0m"


def read(path):
    try:
        return Path(path).read_text().strip()
    except OSError:
        return None


def run(cmd):
    try:
        out = subprocess.run(cmd, capture_output=True, text=True, timeout=5)
        return out.stdout.strip() if out.returncode == 0 else None
    except (OSError, subprocess.SubprocessError):
        return None


def perm_mac(iface):
    """Hardware-burned MAC via ethtool -P (the ground truth for spoof detection)."""
    out = run(["ethtool", "-P", iface])
    if not out:
        return None
    mac = out.split()[-1].lower()
    if mac == "00:00:00:00:00:00" or len(mac) != 17:
        return None
    return mac


def load_backup():
    path = Path.home() / ".local/share/z-hwid/state.json"
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError):
        return {}


def real_interfaces():
    base = Path("/sys/class/net")
    out = []
    if not base.exists():
        return out
    for dev in sorted(p.name for p in base.iterdir()):
        if dev == "lo":
            continue
        d = base / dev
        is_wifi = (d / "wireless").exists() or (d / "phy80211").exists()
        if not is_wifi and not (d / "device").exists():
            continue
        out.append((dev, "wifi" if is_wifi else "ethernet"))
    return out


def status(label, spoofed):
    if spoofed is None:
        return c("? unknown", "dim")
    return c("● SPOOFED", "yellow") if spoofed else c("○ real", "green")


def line(k, v, extra=""):
    v = v if v else c("—", "dim")
    print(f"  {k:<16} {v} {extra}")


def report():
    backup = load_backup()
    active_flag = backup.get("active", False)
    any_spoofed = False

    print(c("\n  Z-HWID · identity check", "bold"))
    if backup:
        state = c("ON", "yellow") if active_flag else c("OFF", "green")
        print(c(f"  backup found · Z-HWID toggle recorded as {state}", "dim"))
    else:
        print(c("  no Z-HWID backup found (spoof state inferred from hardware)", "dim"))

    # ---- network interfaces ----
    print(c("\n  Network (MAC)", "cyan"))
    for iface, kind in real_interfaces():
        cur = read(f"/sys/class/net/{iface}/address")
        perm = perm_mac(iface)
        spoofed = None if not (cur and perm) else (cur.lower() != perm.lower())
        any_spoofed = any_spoofed or bool(spoofed)
        extra = status(iface, spoofed)
        line(f"{iface} ({kind})", cur, extra)
        if perm and spoofed:
            line("", c(f"real {perm}", "dim"))

    # ---- system identity ----
    print(c("\n  System identity", "cyan"))

    # Compared against the backed-up original, so this is definitive whether the
    # toggle is currently ON (differs -> spoofed) or OFF/restored (matches -> real).
    cur_mid = read("/etc/machine-id")
    orig_mid = backup.get("machine_id")
    mid_spoofed = (cur_mid != orig_mid) if (orig_mid and cur_mid) else None
    any_spoofed = any_spoofed or bool(mid_spoofed)
    line("machine-id", cur_mid, status("machine-id", mid_spoofed))
    if mid_spoofed:
        line("", c(f"real {orig_mid}", "dim"))

    cur_host = read("/proc/sys/kernel/hostname")
    orig_host = backup.get("hostname")
    host_spoofed = (cur_host != orig_host) if (orig_host and cur_host) else None
    any_spoofed = any_spoofed or bool(host_spoofed)
    line("hostname", cur_host, status("hostname", host_spoofed))
    if host_spoofed:
        line("", c(f"real {orig_host}", "dim"))

    # ---- DHCP identity (NetworkManager) ----
    if run(["nmcli", "--version"]):
        print(c("\n  DHCP identity", "cyan"), c("(NetworkManager)", "dim"))
        backup_ifaces = {i.get("name"): i for i in backup.get("interfaces", [])}
        for iface, _kind in real_interfaces():
            conn = run(
                ["nmcli", "-t", "-e", "no", "-g", "GENERAL.CONNECTION", "device", "show", iface]
            )
            if not conn or conn == "--":
                continue
            cid = run(
                ["nmcli", "-t", "-e", "no", "-g", "ipv4.dhcp-client-id", "connection", "show", conn]
            ) or ""
            duid = run(
                ["nmcli", "-t", "-e", "no", "-g", "ipv6.dhcp-duid", "connection", "show", conn]
            ) or ""
            b = backup_ifaces.get(iface, {})
            orig_cid = b.get("dhcp_client_id")
            spoofed = (cid != orig_cid) if (orig_cid is not None) else None
            any_spoofed = any_spoofed or bool(spoofed)
            line(f"{iface} client-id", cid or c("(default)", "dim"), status("cid", spoofed))
            if spoofed:
                line("", c(f"real {orig_cid or '(default)'}", "dim"))
            line(f"{iface} duid", duid or c("(default)", "dim"))

    # ---- firmware DMI (read-only; only spoofable per-app via the cloak) ----
    print(c("\n  Firmware / DMI", "cyan"), c("(system-wide read-only)", "dim"))
    dmi_fields = [
        "sys_vendor",
        "product_name",
        "product_uuid",
        "product_serial",
        "board_vendor",
        "board_name",
        "board_serial",
        "bios_version",
        "chassis_serial",
    ]
    for f in dmi_fields:
        val = read(f"/sys/class/dmi/id/{f}")
        if val is None and not os.access(f"/sys/class/dmi/id/{f}", os.R_OK):
            val = c("(root only)", "dim")
        line(f, val)

    # ---- storage serials ----
    print(c("\n  Storage serials", "cyan"))
    serial_paths = list(Path("/sys/block").glob("*/device/serial")) + list(
        Path("/sys/class/nvme").glob("*/serial")
    )
    if not serial_paths:
        line("(none readable)", None)
    for p in sorted(serial_paths):
        dev = p.parent.parent.name if p.parent.name == "device" else p.parent.name
        line(dev, read(p))

    # ---- verdict ----
    print()
    if any_spoofed:
        print(c("  ► SPOOFED — your identity is currently masked.", "yellow"))
    else:
        print(c("  ► NOT SPOOFED — showing your real identity.", "green"))
    print(
        c(
            "  (Firmware/DMI & storage serials are read-only system-wide; they are\n"
            "   only spoofed for apps launched through Z-HWID's App Cloak.)\n",
            "dim",
        )
    )


def main():
    if "--watch" in sys.argv or "-w" in sys.argv:
        import time

        try:
            while True:
                print("\033[2J\033[H", end="")  # clear screen
                report()
                print(c("  (--watch: refreshing every 2s · Ctrl+C to stop)", "dim"))
                time.sleep(2)
        except KeyboardInterrupt:
            print()
    else:
        report()


if __name__ == "__main__":
    main()
