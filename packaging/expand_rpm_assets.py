#!/usr/bin/env python3
"""Expand the packs/ tree into cargo-generate-rpm asset entries.

cargo-generate-rpm does not support directory assets (issue #155), so this
script appends one asset entry per pack file to the [package.metadata.
generate-rpm] table of Cargo.toml. Intended for CI use only; the working
tree Cargo.toml stays clean (the expanded copy is written in place and
discarded by the runner).

Usage: python3 packaging/expand_rpm_assets.py
"""

import os
import sys

MARKER = '{ source = "LICENSE", dest = "/usr/share/licenses/rapm/LICENSE", mode = "644", doc = true },'
DEST_PREFIX = "/usr/share/rusty-agentic-pack-manager/packs/"


def main() -> int:
    if not os.path.isdir("packs"):
        print("packs/ not found; run from the repository root", file=sys.stderr)
        return 1

    entries = []
    for root, _dirs, files in os.walk("packs"):
        for name in sorted(files):
            rel = os.path.relpath(os.path.join(root, name), "packs")
            entries.append(
                f'    {{ source = "packs/{rel}", dest = "{DEST_PREFIX}{rel}", mode = "644" }},'
            )
    if not entries:
        print("no pack files found", file=sys.stderr)
        return 1

    cargo = open("Cargo.toml", encoding="utf-8").read()
    if MARKER not in cargo:
        print("asset marker not found in Cargo.toml", file=sys.stderr)
        return 1
    cargo = cargo.replace(MARKER, MARKER + "\n" + "\n".join(entries))
    open("Cargo.toml", "w", encoding="utf-8").write(cargo)
    print(f"expanded {len(entries)} pack assets into Cargo.toml")
    return 0


if __name__ == "__main__":
    sys.exit(main())
