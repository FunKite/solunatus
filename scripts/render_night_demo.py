#!/usr/bin/env python3
"""Render actual --night output as a dependency-free SVG for the README.

Run after cargo build: python3 scripts/render_night_demo.py --binary target/debug/solunatus
The date and place are deliberately fixed so the preview is reproducible.
"""
import argparse
import html
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/solunatus")
    args = parser.parse_args()
    command = [str(args.binary.resolve()), "--city", "Tucson", "--date", "2026-09-15", "--night", "--no-save"]
    result = subprocess.run(command, check=True, text=True, capture_output=True, timeout=30)
    lines = result.stdout.rstrip().splitlines()
    height = 150 + len(lines) * 26
    svg = [f'''<svg xmlns="http://www.w3.org/2000/svg" width="1120" height="{height}" viewBox="0 0 1120 {height}" role="img" aria-labelledby="title desc">
<title id="title">Solunatus night plan for Tucson on September 15, 2026</title>
<desc id="desc">Real CLI output: evening photography times, a moon-free dark window, and a timed snapshot of Moon and planet positions.</desc>
<rect width="1120" height="{height}" rx="18" fill="#0c1424"/>
<rect x="1" y="1" width="1118" height="{height-2}" rx="18" fill="none" stroke="#30435c"/>
<circle cx="32" cy="30" r="6" fill="#ef8b82"/><circle cx="53" cy="30" r="6" fill="#e9c46a"/><circle cx="74" cy="30" r="6" fill="#81bfa7"/>
<text x="110" y="36" fill="#9cb0c9" font-family="monospace" font-size="16">solunatus · an evening under the stars</text>
<path d="M 22 55 H 1098" stroke="#30435c"/>
<text x="32" y="87" fill="#9adbc6" font-family="monospace" font-size="18">$ solunatus --city Tucson --date 2026-09-15 --night</text>''']
    for i, line in enumerate(lines):
        color = "#e4eaf3"
        if line.isupper() and line:
            color = "#8ccdf5"
        elif line.startswith("Tue "):
            color = "#eacf8e"
        elif line.startswith(("Times are", "Sun below", "America/")):
            color = "#9cb0c9"
        svg.append(f'<text x="32" y="{130+i*26}" xml:space="preserve" fill="{color}" font-family="ui-monospace, SFMono-Regular, Menlo, Consolas, monospace" font-size="20">{html.escape(line)}</text>')
    svg.append("</svg>\n")
    destination = ROOT / "docs/images/night-plan.svg"
    destination.write_text("\n".join(svg))
    print(destination)


if __name__ == "__main__":
    main()
