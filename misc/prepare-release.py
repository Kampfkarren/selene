#!/usr/bin/python3
from datetime import datetime
import os
import re
import sys

if len(sys.argv) != 2:
    print("Usage: prepare-release.py <version>")
    sys.exit(1)

version = sys.argv[1]

print("Updating CHANGELOG...")

with open("CHANGELOG.md", "r") as changelog_file:
    changelog = changelog_file.read()

    changelog = re.sub(
        r"## \[Unreleased\]\((.+)\)$",
        f"## [Unreleased](https://github.com/Kampfkarren/selene/compare/{version}...HEAD)\n\n"
        f"## [{version}](https://github.com/Kampfkarren/selene/releases/tag/{version}) - {datetime.today().strftime('%Y-%m-%d')}",
        changelog,
        0,
        re.MULTILINE
    )

with open("CHANGELOG.md", "w") as changelog_file:
    changelog_file.write(changelog)

print("Updating root Cargo.toml...")

with open("Cargo.toml", "r") as cargo_file:
    cargo = cargo_file.read()

    cargo = re.sub(
        r"version = \"(.+)\"",
        f"version = \"{version}\"",
        cargo,
        0,
        re.MULTILINE
    )

with open("Cargo.toml", "w") as cargo_file:
    cargo_file.write(cargo)

print("Updating selene/Cargo.toml...")

with open("selene/Cargo.toml", "r") as cargo_file:
    selene_cargo = cargo_file.read()

    selene_cargo = re.sub(
        r"^(selene-lib = .+version = \")(=.+?)\"",
        f"\\1={version}\"",
        selene_cargo,
        0,
        re.MULTILINE
    )

with open("selene/Cargo.toml", "w") as cargo_file:
    cargo_file.write(selene_cargo)

print("Running cargo check...")

os.system("cargo check")

print("------")

print("Done. Next steps:")
print("- Make and push a commit with [release]")
print("- cargo publish")
