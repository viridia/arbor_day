"""A gallery of shapes, for the person reading the run: compare the screenshots. The generator's
unit tests check the counts, so this plan waits a few frames after each edit rather than checking
the plant. A section that edits the first level puts it back, so later shots compare with the
default.
- crown softness: a second level for a fuller crown, its cards shading as themselves, then half
  way to the envelope's normal, then all the way;
- node start: the first level's nodes in the upper half of the trunk inside the crown;
- limbs per node and density: whorls, and a mix of one and two limbs to a node;
- wander, turned on for the shots: the first level's limbs zigzag about their course at the most
  wander, 45°, and the second level's shorter twigs kink once or not at all;
- facing: the second level's limbs to the sides (the default), up, down and around, then skewed;
- bias: the second level's twigs longer at each limb's first node and shorter toward its tip, 160%
  falling to 40%, then the other way, short near each limb's root;
- aim: the first level's limbs turned toward the crown's outline, the upper ones toward its top,
  half way, then all the way;
- forks: every first-level limb forking, the Y level, then upright, then three ways, the second
  level to the sides, so the level Y shows whether twigs crowd its crotch;
- stems: the trunk forked at the ground into three stems splayed from up, then a crown twice the
  trunk's height, which leaves the stems in its narrow foot, bearing fewer limbs, then the trunk
  split into two at 40% of its height, a codominant fork."""

import math

BIN = "arbor_day"
SPECIES = "arbor_day::species::Species"


def add_level(driver, last):
    """Shows the level after `last`, the last shown, from its header's add menu."""
    driver.click(f"Shell/Panel/levels[{last}]/add_operator")
    driver.click(f"Shell/Panel/levels[{last}]/add_operator/add")


def set_number(driver, path, text, input="input"):
    """Replaces the text of a row's number input, named `input`, and lets the plant rebuild. A
    click places the cursor rather than selecting, so clear what is there first."""
    driver.click(f"Shell/Panel/{path}/{input}")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type(text)
    driver.key("Enter")
    driver.frames(2)


def pick_facing(driver, variant):
    """Opens the second level's facing select, picks a variant, and lets the plant rebuild."""
    driver.click("Shell/Panel/levels[1]/facing/select")
    driver.click(f"Shell/Panel/levels[1]/facing/{variant}")
    driver.frames(2)


def run(driver):
    driver.present("Shell")  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/levels[0]/add")  # the immediate-mode panel has built its rows
    driver.frames(2)
    driver.screenshot("default")

    add_level(driver, 0)
    driver.frames(2)
    for softness in [0, 50, 100]:
        driver.set_resource(SPECIES, softness / 100, at="ornaments.tip_leaves.settings.softness")
        driver.frames(2)
        driver.screenshot(f"softness_{softness}")
    driver.set_resource(SPECIES, 0.0, at="ornaments.tip_leaves.settings.softness")
    driver.click("Shell/Panel/levels[1]/remove")
    driver.frames(2)

    set_number(driver, "levels[0]/start", "50")
    driver.screenshot("start_halfway")
    set_number(driver, "levels[0]/start", "0")

    set_number(driver, "levels[0]/per_node", "3")
    driver.screenshot("whorled")
    set_number(driver, "levels[0]/density", "2")
    set_number(driver, "levels[0]/per_node", "1.5")
    driver.screenshot("mixed")
    set_number(driver, "levels[0]/density", "8")
    set_number(driver, "levels[0]/per_node", "1")

    driver.set_resource(SPECIES, "On", at="limbs.levels[0].wander.state")
    set_number(driver, "levels[0]/wander/angle", "45")
    driver.screenshot("most_wander")
    add_level(driver, 0)
    driver.set_resource(SPECIES, "On", at="limbs.levels[1].wander.state")
    driver.frames(2)
    driver.screenshot("most_wander_twigs")
    set_number(driver, "levels[0]/wander/angle", "10")
    driver.set_resource(SPECIES, "Off", at="limbs.levels[0].wander.state")
    driver.set_resource(SPECIES, "Off", at="limbs.levels[1].wander.state")

    driver.screenshot("facing_sides")
    for variant in ["Up", "Down", "Around"]:
        pick_facing(driver, variant)
        driver.screenshot(f"facing_{variant.lower()}")
    # The most skew turns each limb up to 90° from where its facing put it.
    set_number(driver, "levels[1]/skew", "90")
    driver.screenshot("skewed")
    set_number(driver, "levels[1]/skew", "20")
    pick_facing(driver, "Sides")

    # Twigs longer low on each limb and shorter toward its tip, as a conifer's; then the other
    # way, short near each limb's root.
    driver.set_resource(SPECIES, "On", at="limbs.levels[1].bias.state")
    size = "limbs.levels[1].bias.settings.size.0"
    driver.set_resource(SPECIES, [[0.0, 1.6], [1.0, 0.4]], at=size)
    driver.frames(2)
    driver.screenshot("bias")
    driver.set_resource(SPECIES, [[0.0, 0.4], [1.0, 1.6]], at=size)
    driver.frames(2)
    driver.screenshot("bias_reversed")
    driver.set_resource(SPECIES, "Off", at="limbs.levels[1].bias.state")

    # The first level's limbs aimed at the crown's outline, half way, then all the way.
    driver.set_resource(SPECIES, "On", at="limbs.levels[0].bias.state")
    set_number(driver, "levels[0]/bias/aim", "50")
    driver.screenshot("aimed_half")
    set_number(driver, "levels[0]/bias/aim", "100")
    driver.screenshot("aimed")
    driver.set_resource(SPECIES, "Off", at="limbs.levels[0].bias.state")

    # Every first-level limb forks: level, upright, then three ways.
    driver.set_resource(SPECIES, "On", at="limbs.levels[0].fork.state")
    set_number(driver, "levels[0]/fork/chance", "100")
    driver.set_resource(SPECIES, 0.0, at="limbs.levels[0].fork.settings.roll.spread")
    driver.screenshot("forked")
    driver.set_resource(SPECIES, math.radians(90), at="limbs.levels[0].fork.settings.roll.center")
    driver.frames(2)
    driver.screenshot("forked_upright")
    driver.set_resource(SPECIES, 0.0, at="limbs.levels[0].fork.settings.roll.center")
    set_number(driver, "levels[0]/fork/copies", "3")
    driver.screenshot("forked_three")
    set_number(driver, "levels[0]/fork/copies", "2")
    set_number(driver, "levels[0]/fork/chance", "0")
    driver.set_resource(SPECIES, "Off", at="limbs.levels[0].fork.state")
    driver.click("Shell/Panel/levels[1]/remove")
    driver.frames(2)

    # The trunk forked at the ground into three stems 25° from up, then a tall crown, then the
    # trunk split into two partway up, under the default crown.
    driver.set_resource(SPECIES, "On", at="trunk.fork.state")
    driver.set_resource(SPECIES, 0.0, at="trunk.fork.settings.at.center")
    driver.set_resource(SPECIES, 0.0, at="trunk.fork.settings.at.spread")
    driver.set_resource(SPECIES, math.radians(50), at="trunk.fork.settings.angle.center")
    driver.set_resource(SPECIES, 0.0, at="trunk.fork.settings.angle.spread")
    set_number(driver, "trunk/fork/copies", "3")
    driver.screenshot("three_stems")
    set_number(driver, "crown/top", "200")
    driver.screenshot("tall_crown")
    set_number(driver, "crown/top", "125")
    driver.set_resource(SPECIES, 0.4, at="trunk.fork.settings.at.center")
    set_number(driver, "trunk/fork/copies", "2")
    driver.screenshot("split_trunk")
    driver.exit()
