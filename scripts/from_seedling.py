"""Converts seedling models to arbor_day species files, best effort, as starting points to finish
by hand in arbor_day (C131).

    python3 scripts/from_seedling.py SEEDLING_DIR OUT_DIR [NAME...]

Each model under `SEEDLING_DIR` (Panoply's `artwork/flora`), or only those named, without
`.glb`, has its `sapling` extra written as a species to `OUT_DIR`, in the same subfolder and
under the same name, as a `.gltf`: a glTF with no meshes, holding the species as arbor_day's
scene extra. Opening it in arbor_day and saving fills in the meshes. The crown, which seedling
did not have, is measured from the model's leaves. What has no counterpart is printed, model by
model.
"""

import json
import math
import struct
import sys
from pathlib import Path

# arbor_day's species-file version (`species_file::GENERATOR`).
GENERATOR = 1

# arbor_day's `species::MAX_LIMB_LEVELS`.
MAX_LIMB_LEVELS = 3


def read_glb(path):
    """The glTF JSON and binary chunk of the `.glb` at `path`."""
    data = path.read_bytes()
    json_length = struct.unpack_from("<I", data, 12)[0]
    root = json.loads(data[20 : 20 + json_length])
    bin_start = 20 + json_length
    bin_length = struct.unpack_from("<I", data, bin_start)[0]
    return root, data[bin_start + 8 : bin_start + 8 + bin_length]


def positions(root, binary, node_name):
    """The vertex positions of the node named `node_name`'s mesh, as (x, y, z) tuples."""
    node = next(node for node in root["nodes"] if node.get("name") == node_name)
    if any(key in node for key in ("matrix", "translation", "rotation", "scale")):
        raise ValueError(f"node {node_name} has a transform, which is not applied")
    mesh = root["meshes"][node["mesh"]]
    accessor = root["accessors"][mesh["primitives"][0]["attributes"]["POSITION"]]
    view = root["bufferViews"][accessor["bufferView"]]
    offset = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
    stride = view.get("byteStride", 12)
    return [
        struct.unpack_from("<3f", binary, offset + i * stride) for i in range(accessor["count"])
    ]


def rgb(text):
    """Seedling's `rgb(r,g,b)` as an arbor_day color."""
    red, green, blue = (int(part) / 255 for part in text[4:-1].split(","))
    return {"Srgba": {"red": red, "green": green, "blue": blue, "alpha": 1.0}}


def mid(pair):
    return (pair[0] + pair[1]) / 2


def uniform(pair, scale=1.0):
    """Seedling's range as a uniform fuzzy value, times `scale`."""
    return {
        "center": mid(pair) * scale,
        "spread": (pair[1] - pair[0]) / 2 * scale,
        "shape": "Uniform",
    }


def clamp(value, low, high):
    return min(max(value, low), high)


def median(values):
    values = sorted(values)
    return values[len(values) // 2]


def measure(leaves, lateral_droop):
    """The crown's widest radius, its lowest and highest tip, its profile, and a card's side, from
    the leaf mesh: each card is 3 rows of 4 vertices, base to tip, the middle two at the midrib,
    its middle row at the twig's tip."""
    cards = [leaves[card * 12 : card * 12 + 12] for card in range(len(leaves) // 12)]
    tips = [card[5] for card in cards]
    radius = max(math.hypot(x, z) for x, _, z in tips)
    low = min(y for _, y, _ in tips)
    high = max(y for _, y, _ in tips)
    sides = []
    for card in cards:
        rows = [card[row * 4 : row * 4 + 4] for row in range(3)]
        width = math.dist(rows[0][0], rows[0][3]) / max(math.cos(lateral_droop), 0.1)
        length = math.dist(rows[0][1], rows[1][1]) + math.dist(rows[1][1], rows[2][1])
        sides.append(max(width, length))
    return radius, low, high, profile(tips, low, high), median(sides)


def profile(tips, low, high):
    """The crown's profile by where the tips spread widest, in thirds of their height: the same
    in each, a column; the lowest, a cone, or a hemisphere if the top third is still wide; the
    middle, an ellipsoid; the top, a flame."""
    widths = [0.0, 0.0, 0.0]
    for x, y, z in tips:
        third = min(int((y - low) / max(high - low, 1e-6) * 3), 2)
        widths[third] = max(widths[third], math.hypot(x, z))
    widest = max(widths)
    if min(widths) > 0.8 * widest:
        return "Column"
    third = widths.index(widest)
    if third == 0:
        return "Hemisphere" if widths[2] > 0.6 * widest else "Cone"
    return "Ellipsoid" if third == 1 else "Flame"


def sag_or_rise(level, flex, crown_radius):
    """Seedling's flex turned a limb down by 4 × flex × sin(angle from up) radians a metre: as
    Sag over a crown radius when positive, or Rise's share of the angle when negative."""
    turn = 4 * abs(flex) * crown_radius
    if flex > 0:
        level["sag"] = {"amount": clamp(turn, 0.0, math.pi)}
    elif flex < 0:
        level["rise"] = {"amount": clamp(turn * 2 / math.pi, 0.0, 0.99)}


def monopodial(sapling, crown_radius, notes):
    """Each branch group as a limb level: nodes from its interval, limbs per node from its
    symmetry, and its length taper as Bias's size."""
    levels = []
    for index, group in enumerate(sapling["branch"][:MAX_LIMB_LEVELS]):
        axis_turn = mid(group["axis"])
        level = {
            "density": clamp(crown_radius / max(mid(group["interval"]), 0.01), 0.0, 40.0),
            "per_node": clamp(group["symmetry"], 1, 6),
            "facing": "Sides" if abs(axis_turn) < 0.1 else "Around",
            "skew": clamp((group["axis"][1] - group["axis"][0]) / 2, 0.0, math.pi / 2),
            "start": clamp(mid(group["branchAt"]), 0.0, 0.9),
            "angle": uniform(group["angle"]),
        }
        sag_or_rise(level, group.get("flex", 0), crown_radius)
        # Seedling's angle bias was added to the angle from the first node to the last. Negative,
        # the upper limbs steeper, is Aim's turn toward the crown's top, by the share of the angle
        # it closed.
        length_taper = group.get("lengthTaper", 1)
        angle_bias = group.get("angleBias", 0)
        aim = clamp(-angle_bias / max(mid(group["angle"]), 0.01), 0.0, 1.0)
        if length_taper > 0 or aim > 0:
            level["bias"] = {"size": length_taper / (2 - length_taper), "aim": aim}
        if angle_bias > 0:
            notes.append(f"branch[{index}].angleBias {angle_bias}: no counterpart when positive")
        levels.append(level)
    return levels


def dichotomous(sapling, trunk, crown_radius, notes):
    """The first branch group as the trunk's fork, at the old trunk's tip; each later one as a
    limb level of one node, its limbs the fork's side branches, the parent carrying on as the
    third."""
    groups = sapling["branch"]
    first = groups[0]
    length = sapling["trunk"]["length"]
    trunk["height"] = clamp(length + mid(first["length"]), 0.1, 30.0)
    if first["symmetry"] > 1:
        trunk["fork"] = {
            "chance": 1.0,
            "at": {"center": clamp(length / trunk["height"], 0.0, 0.95), "spread": 0.0,
                   "shape": "Uniform"},
            "copies": clamp(first["symmetry"], 2, 5),
            "angle": uniform(first["angle"], 2.0),
            "roll": {"center": 0.0, "spread": 0.0, "shape": "Uniform"},
        }
    if first.get("flex", 0):
        notes.append(f"branch[0].flex {first['flex']}: the trunk has no sag or rise")
    levels = []
    for index, group in enumerate(groups[1 : MAX_LIMB_LEVELS + 1], start=1):
        parent_length = mid(groups[index - 1]["length"])
        level = {
            "density": clamp(crown_radius / max(parent_length, 0.01), 0.0, 40.0),
            "per_node": clamp(group["symmetry"] - 1, 1, 6),
            "facing": "Around",
            "start": 0.9,
            "end": 1.0,
            "angle": uniform(group["angle"]),
        }
        sag_or_rise(level, group.get("flex", 0), crown_radius)
        levels.append(level)
    if not levels:
        notes.append("one branch group: no limb levels; arbor_day needs one, left at its default")
    notes.append("forks happen once in arbor_day (C132); later forks are limbs near each tip")
    return levels


def species(sapling, leaves, notes):
    """The species file for one seedling model."""
    old_trunk = sapling["trunk"]
    group = sapling["leafGroup"]
    shape = sapling["leafShape"]
    color = sapling["leafColor"]
    crown_radius, low, high, crown_profile, card = measure(leaves, group["lateralDroop"])

    trunk = {
        "height": clamp(old_trunk["length"], 0.1, 30.0),
        "radius": clamp(old_trunk["radius"], 0.02, 1.0),
        "sides": 6,
    }
    pattern = old_trunk["growthPattern"]
    if pattern == "monopodial":
        levels = monopodial(sapling, crown_radius, notes)
    else:
        levels = dichotomous(sapling, trunk, crown_radius, notes)
    # The crown's base no higher than the first limbs, or the fork, so they grow inside it.
    height = trunk["height"]
    base = low / height
    if pattern == "monopodial" and sapling["branch"]:
        base = min(base, mid(sapling["branch"][0]["branchAt"]))
    elif "fork" in trunk:
        base = min(base, trunk["fork"]["at"]["center"])

    file = {
        "generator": GENERATOR,
        # Seedling's taper was the radius left at an average tip; arbor_day's is the share lost.
        "taper": clamp(1 - old_trunk["taper"], 0.0, 1.0),
        "bark_color": rgb(old_trunk["color"]),
        "trunk": trunk,
        "crown": {
            "profile": crown_profile,
            "top": clamp(high / height, 1.0, 2.0),
            "base": clamp(base, 0.0, 0.95),
            "radius": clamp(crown_radius, 0.1, 15.0),
        },
    }
    if levels:
        file["limbs"] = levels
    fan = group["fanSize"] - 1
    file["ornaments"] = {
        "tip_leaves": {
            "upright": "Triple",
            "size": clamp(card / crown_radius, 0.02, 1.0),
            "lateral_droop": group["lateralDroop"],
            "axial_droop": group["axialDroop"],
            "sprig": {
                "fan": clamp(fan, 0, 9),
                "angle": {"center": group["fanAngle"], "spread": group["angleVariation"] / 2},
                "outer_size": clamp(group["taper"], 0.0, 2.0),
                "pairs": clamp(group["phalanxCount"], 0, 10),
                # Seedling's spacing was in texture pixels, its leaf `length` pixels long.
                "spacing": {
                    "center": clamp(group["leafSpacing"] / shape["length"], 0.0, 1.5),
                    "spread": group["leafSpacing"] * group["leafSpacingVariation"] / 2
                    / shape["length"],
                },
            },
            "leaf": {
                "base_width": shape["baseWidth"],
                "base_taper": shape["baseTaper"],
                "tip_width": shape["tipWidth"],
                "tip_taper": shape["tipTaper"],
                "segments": shape["numSegments"],
                "serration": shape["serration"],
                "rake": shape["rake"],
                "jitter": shape["jitter"],
                "inner_color": rgb(color["innerColor"]),
                "outer_color": rgb(color["outerColor"]),
                "gradient": color["gradientDirection"].capitalize(),
                "variation": color["variation"],
            },
        }
    }
    if len(sapling["branch"]) > MAX_LIMB_LEVELS + (pattern != "monopodial"):
        notes.append(f"{len(sapling['branch'])} branch groups: past {MAX_LIMB_LEVELS} dropped")
    return rounded(file)


def rounded(value, path=()):
    """`value` with each number rounded to the precision the panel shows it in: angles to a tenth
    of a degree, lengths in metres and counts by their field, and the rest, shares, to a whole
    percent. A fuzzy value's center and spread are rounded as their field is."""
    if isinstance(value, dict):
        return {key: rounded(item, path + (key,)) for key, item in value.items()}
    if isinstance(value, list):
        return [rounded(item, path + (f"[{index}]",)) for index, item in enumerate(value)]
    if not isinstance(value, float) or "Srgba" in path:
        return value
    *parents, name = path
    if name in ("center", "spread"):
        *parents, name = parents
    parent = parents[-1] if parents else ""
    if name in ("skew", "angle", "lateral_droop", "axial_droop") or parent == "sag":
        return math.radians(round(math.degrees(value), 1))
    if name == "height" or (parent, name) == ("trunk", "radius"):
        return round(value, 2)
    if name in ("density", "per_node") or (parent, name) == ("crown", "radius"):
        return round(value, 1)
    return round(value, 2)


def gltf(name, species_file):
    """An empty glTF holding `species_file` as arbor_day's scene extra (`gltf_file::SPECIES_KEY`)."""
    return {
        "asset": {"version": "2.0", "generator": "arbor_day scripts/from_seedling.py"},
        "scene": 0,
        "scenes": [{"name": name, "nodes": [], "extras": {"arbor_day": species_file}}],
    }


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    seedling_dir, out_dir, names = Path(sys.argv[1]), Path(sys.argv[2]), set(sys.argv[3:])
    models = sorted(seedling_dir.rglob("*.glb"))
    missing = names - {model.stem for model in models}
    if missing:
        sys.exit(f"no such model under {seedling_dir}: {', '.join(sorted(missing))}")
    for model in models:
        if names and model.stem not in names:
            continue
        root, binary = read_glb(model)
        sapling = root["scenes"][root.get("scene", 0)]["extras"]["sapling"]
        notes = []
        file = species(sapling, positions(root, binary, "leaf"), notes)
        out = (out_dir / model.relative_to(seedling_dir)).with_suffix(".gltf")
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(json.dumps(gltf(model.stem, file), indent=2) + "\n")
        print(f"{out}")
        for note in notes:
            print(f"  {note}")


if __name__ == "__main__":
    main()
