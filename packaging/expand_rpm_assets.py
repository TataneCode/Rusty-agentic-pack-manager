#!/usr/bin/env python3
"""Expand the packs/ tree into cargo-generate-rpm and cargo-deb asset entries.

cargo-generate-rpm does not support directory assets (issue #155) and the
cargo-deb glob does not recurse with **, so this script appends one asset
entry per pack file to both the [package.metadata.generate-rpm] and
[package.metadata.deb] tables of Cargo.toml. Intended for CI use only; the
working tree Cargo.toml stays clean (the expanded copy is written in place
and discarded by the runner).

Usage: python3 packaging/expand_rpm_assets.py
"""

import os
import sys

RPM_MARKER = '{ source = "LICENSE", dest = "/usr/share/licenses/rapm/LICENSE", mode = "644", doc = true },'
DEB_MARKER = '["LICENSE", "usr/share/doc/rusty-agentic-pack-manager/LICENSE", "644"],'
DEST_PREFIX = "/usr/share/rusty-agentic-pack-manager/packs/"


def main() -> int:
    if not os.path.isdir("packs"):
        print("packs/ not found; run from the repository root", file=sys.stderr)
        return 1

    rpm_entries = []
    deb_entries = []
    count = 0
    for root, _dirs, files in os.walk("packs"):
        for name in sorted(files):
            rel = os.path.relpath(os.path.join(root, name), "packs")
            rpm_entries.append(
                f'    {{ source = "packs/{rel}", dest = "{DEST_PREFIX}{rel}", mode = "644" }},'
            )
            deb_entries.append(
                f'    ["packs/{rel}", "usr/share/rusty-agentic-pack-manager/packs/{rel}", "644"],'
            )
            count += 1
    if not count:
        print("no pack files found", file=sys.stderr)
        return 1

    cargo = open("Cargo.toml", encoding="utf-8").read()
    if RPM_MARKER not in cargo or DEB_MARKER not in cargo:
        print("asset marker not found in Cargo.toml", file=sys.stderr)
        return 1
    cargo = cargo.replace(RPM_MARKER, RPM_MARKER + "\n" + "\n".join(rpm_entries))
    cargo = cargo.replace(DEB_MARKER, DEB_MARKER + "\n" + "\n".join(deb_entries))
    open("Cargo.toml", "w", encoding="utf-8").write(cargo)
    print(f"expanded {count} pack assets (rpm + deb) into Cargo.toml")
    return 0


if __name__ == "__main__":
    sys.exit(main())
