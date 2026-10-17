"""A gallery of shapes, for the person reading the run: compare the screenshots. The generator's
unit tests check the counts, so this plan waits a few frames after each edit rather than checking
the plant. A section that edits the first level puts it back, so later shots compare with the
default.
- node start: the first level's nodes in the upper half of the trunk inside the crown;
- limbs per node and density: whorls, and a mix of one and two limbs to a node;
- wander: the first level's limbs zigzag about their course at the most wander, 45°, and the
  second level's shorter twigs kink once or not at all;
- facing: the second level's limbs to the sides (the default), up, down and around, then skewed;
- forks: every first-level limb forking, the Y level, then upright, then three ways, the second
  level to the sides, so the level Y shows whether twigs crowd its crotch (ledger X33);
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


def set_number(driver, path, text):
    """Replaces a number input's text and lets the plant rebuild. A click places the cursor rather
    than selecting, so clear what is there first."""
    driver.click(f"Shell/Panel/{path}/input")
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
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/levels[0]/add")  # the immediate-mode panel has built its rows
    driver.frames(2)
    driver.screenshot("default")

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

    set_number(driver, "levels[0]/wander/angle", "45")
    driver.screenshot("most_wander")
    add_level(driver, 0)
    driver.frames(2)
    driver.screenshot("most_wander_twigs")
    set_number(driver, "levels[0]/wander/angle", "10")

    driver.screenshot("facing_sides")
    for variant in ["Up", "Down", "Around"]:
        pick_facing(driver, variant)
        driver.screenshot(f"facing_{variant.lower()}")
    # The most skew turns each limb up to 90° from where its facing put it.
    set_number(driver, "levels[1]/skew", "90")
    driver.screenshot("skewed")
    set_number(driver, "levels[1]/skew", "20")
    pick_facing(driver, "Sides")

    # Every first-level limb forks: level, upright, then three ways.
    driver.set_resource(SPECIES, "On", at="limbs.levels[0].fork.state")
    set_number(driver, "levels[0]/fork/chance", "100")
    driver.screenshot("forked")
    set_number(driver, "levels[0]/fork/roll", "90")
    driver.screenshot("forked_upright")
    set_number(driver, "levels[0]/fork/roll", "0")
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
