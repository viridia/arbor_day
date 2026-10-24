"""Leechgrass and structural variants of it, each shot low from the front and from above, with the
scale shown, for tuning by eye."""

BIN = "arbor_day"
REQUEST = "arbor_day::document::FileRequest"
SPECIES = "arbor_day::species::Species"
TEXT = "bevy_ui::widget::text::Text"
TURNTABLE = "arbor_day::turntable::TurntableCamera"
NAME = "Shell/Panel/document/name"
PATH = "/Users/talin/Projects/games/arbor_day/reference/shrubs/leechgrass.gltf"

# A short trunk, limbs in whorls on its lower part, leaving at a wide spread of angles.
WHORLS = {
    "trunk.height": 0.25,
    "trunk.fork.state": "Off",
    "crown.base": 0.0,
    "crown.top": 1.8,
    "crown.radius": 0.45,
    "limbs.shown": 1,
    "limbs.levels[0].start": 0.0,
    "limbs.levels[0].end": 0.4,
    "limbs.levels[0].density": 20.0,
    "limbs.levels[0].per_node": 3.0,
    "limbs.levels[0].skew": 0.3,
    "limbs.levels[0].reach.center": 0.9,
    "limbs.levels[0].reach.spread": 0.1,
}

# Seedling's clump-green: a few spokes from the ground, each forking toward its end into twigs
# ending in long sprays.
SPOKES = {
    "trunk.height": 0.3,
    "trunk.fork.state": "On",
    "trunk.fork.settings.at.center": 0.0,
    "trunk.fork.settings.copies": 5,
    "trunk.fork.settings.angle.shape": "Uniform",
    "trunk.fork.settings.angle.center": 2.0,
    "trunk.fork.settings.angle.spread": 0.3,
    "trunk.fork.settings.roll.spread": 0.6,
    "crown.base": 0.0,
    "crown.top": 1.5,
    "crown.radius": 0.45,
    "limbs.shown": 1,
    "limbs.levels[0].start": 0.5,
    "limbs.levels[0].end": 1.0,
    "limbs.levels[0].density": 4.0,
    "limbs.levels[0].per_node": 1.0,
    "limbs.levels[0].angle.center": 0.6,
    "limbs.levels[0].angle.spread": 0.15,
    "limbs.levels[0].reach.center": 0.6,
    "limbs.levels[0].reach.spread": 0.15,
}

# Narrow pointed leaves, a long spray of them.
BLADES = {
    "ornaments.tip_leaves.settings.size": 0.6,
    "ornaments.tip_leaves.settings.leaf.base_width": 0.1,
    "ornaments.tip_leaves.settings.leaf.tip_width": 0.05,
    "ornaments.tip_leaves.settings.sprig.angle.center": 0.6,
}

VARIANTS = {
    "spokes": SPOKES,
    "spokes-blades": {**SPOKES, **BLADES},
}

OLD_VARIANTS = {
    "aloe": {
        **WHORLS,
        "limbs.levels[0].angle.shape": "Uniform",
        "limbs.levels[0].angle.center": 0.96,
        "limbs.levels[0].angle.spread": 0.44,
        "limbs.levels[0].rise.state": "On",
        "limbs.levels[0].rise.settings.amount": 0.3,
    },
    "spider": {
        **WHORLS,
        "limbs.levels[0].angle.shape": "Uniform",
        "limbs.levels[0].angle.center": 1.2,
        "limbs.levels[0].angle.spread": 0.3,
        "limbs.levels[0].rise.state": "Off",
        "limbs.levels[0].sag.state": "On",
        "limbs.levels[0].sag.settings.amount": 0.6,
    },
    # Five stems from the ground, each a bristly stalk.
    "clump": {
        "trunk.height": 0.35,
        "trunk.fork.state": "On",
        "trunk.fork.settings.at.center": 0.0,
        "trunk.fork.settings.copies": 5,
        "trunk.fork.settings.angle.shape": "Uniform",
        "trunk.fork.settings.angle.center": 1.75,
        "trunk.fork.settings.angle.spread": 0.5,
        "trunk.fork.settings.roll.spread": 0.6,
        "crown.base": 0.0,
        "crown.top": 1.3,
        "crown.radius": 0.45,
        "limbs.shown": 1,
        "limbs.levels[0].start": 0.3,
        "limbs.levels[0].end": 1.0,
        "limbs.levels[0].density": 8.0,
        "limbs.levels[0].per_node": 2.0,
        "limbs.levels[0].angle.center": 0.7,
        "limbs.levels[0].angle.spread": 0.2,
        "limbs.levels[0].reach.center": 0.45,
        "limbs.levels[0].reach.spread": 0.15,
    },
}


def shoot(driver, name):
    driver.click("Shell/View/frame")
    driver.expect("SceneCamera", TURNTABLE, {"frame_pending": False})
    driver.set("SceneCamera", TURNTABLE, 0.15, at="elevation")
    driver.frames(2)
    driver.screenshot(f"{name}-low")
    driver.set("SceneCamera", TURNTABLE, 1.45, at="elevation")
    driver.frames(2)
    driver.screenshot(f"{name}-above")


def run(driver):
    driver.present("Shell", ready=True)
    driver.present("Shell/Panel/levels[0]/add")
    driver.click("Shell/View/scale")
    driver.message(REQUEST, {"Open": PATH})
    driver.expect(NAME, TEXT, "leechgrass")
    shoot(driver, "current")
    for name, fields in VARIANTS.items():
        driver.message(REQUEST, {"Open": PATH})
        driver.frames(2)
        for path, value in fields.items():
            driver.set_resource(SPECIES, value, at=path)
        driver.frames(2)
        shoot(driver, name)
    driver.exit()
