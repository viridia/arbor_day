"""Converts species files from before Bias's Size became a curve (C145): each Bias `size`, a number
`s`, becomes the curve from 1 + s at the first node to 1 - s at the last, written
`[[0, 1 + s], [1, 1 - s]]`, which grows the same tree.

    python3 scripts/size_curve.py [PATH...]

Each `.gltf` under each path, `reference/` by default, is rewritten in place if it holds an old
Size, in its own layout: arbor_day's, the species extra on one line, or that of
`scripts/from_seedling.py`, indented throughout. A file already converted is left as it is.
"""

import json
import re
import sys
from pathlib import Path

# A Bias in arbor_day's one-line layout, holding no nested object or list.
BIAS = re.compile(r'"bias":(\{[^{}\[\]]*\})')


def size_curve(change):
    """The curve from `1 + change` at the first node to `1 - change` at the last, as arbor_day's
    `PlaceCurve` writes it: `[place, value]` pairs."""
    return [[0.0, round(1 + change, 6)], [1.0, round(1 - change, 6)]]


def convert_bias(bias):
    """`bias` with its Size as a curve."""
    if isinstance(bias.get("size"), (int, float)):
        bias["size"] = size_curve(bias["size"])
    return bias


def convert_tree(value):
    """Every Bias under `value` converted."""
    if isinstance(value, dict):
        return {
            key: convert_bias(item) if key == "bias" else convert_tree(item)
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [convert_tree(item) for item in value]
    return value


def convert_text(text):
    """`text` with each Bias converted, its layout kept."""
    root = json.loads(text)
    if json.dumps(root, indent=2) + "\n" == text:
        return json.dumps(convert_tree(root), indent=2) + "\n"

    def compact(match):
        bias = convert_bias(json.loads(match.group(1)))
        return '"bias":' + json.dumps(bias, sort_keys=True, separators=(",", ":"))

    return BIAS.sub(compact, text)


def main():
    roots = [Path(arg) for arg in sys.argv[1:]] or [Path("reference")]
    for root in roots:
        files = [root] if root.is_file() else sorted(root.rglob("*.gltf"))
        for path in files:
            text = path.read_text()
            converted = convert_text(text)
            if converted != text:
                path.write_text(converted)
                print(path)


if __name__ == "__main__":
    main()
