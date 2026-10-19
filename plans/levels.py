"""Limb levels and foliage. The last level's add menu shows the next, growing on the limbs of the
one before, and a level's ✕ hides it and those after it. A hidden level keeps its parameters, so
showing it again brings it back as it was. A leaf card grows at each branch end, and the tip
leaves' ✕ leaves the tree bare until Foliage's add menu brings them back.

Then, from a new document, card folds: the lateral droop turns each half of a card down about its
midrib, or up, cupping it, when negative; the axial droop bends its far half down. Checked by eye
in the screenshots. Upright tips bear a rosette, one card each, or none."""

import math

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"
GROWN_FROM = "arbor_day::plant::GrownFrom"
SPECIES = "arbor_day::species::Species"
VISIBILITY = "bevy_camera::visibility::Visibility"
TIP_LEAVES = "ornaments.tip_leaves.settings"


def add_level(driver, last):
    """Shows the level after `last`, the last shown, from its header's add menu."""
    driver.click(f"Shell/Panel/levels[{last}]/add_operator")
    driver.click(f"Shell/Panel/levels[{last}]/add_operator/add")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/levels[0]/add")  # the immediate-mode panel has built its rows
    levels(driver)
    driver.click("Shell/Panel/document/file_menu")
    driver.click("Shell/Panel/document/file_menu/new")
    cards(driver)
    driver.exit()


def levels(driver):
    # Nine or ten, each bearing a card at its tip, none of them steep, and the trunk's upright
    # tip a rosette of three.
    first = Between(9, 10)
    leaves = {"leafy_tips": Between(10, 11), "cards": Between(9 + 3, 10 + 3)}
    driver.expect("Plant", STATS, {"limbs": first, "level_limbs": [first], **leaves})
    add_level(driver, 0)
    # One to six nodes of one or two on each of the first level's nine or ten limbs, 1 to 3 m
    # long. Each level's count shows in its header.
    second = Between(9 * 2, 10 * 2 * 6)
    driver.expect("Plant", STATS, {"level_limbs": [first, second]})
    driver.absent("Shell/Panel/levels[0]/add")  # the next level is offered by the last only
    driver.absent("Shell/Panel/levels[0]/remove")  # the first level stays
    driver.screenshot("two_levels")

    # Fewer nodes: up to two on each first-level limb, and some bearing none.
    driver.set_resource(SPECIES, 1.5, at="limbs.levels[1].density")
    fewer = {"level_limbs": [first, Between(5, 10 * 2 * 2)]}
    driver.expect("Plant", STATS, fewer)

    driver.click("Shell/Panel/levels[1]/remove")
    driver.expect("Plant", STATS, {"level_limbs": [first]})
    driver.absent("Shell/Panel/levels[1]")
    add_level(driver, 0)
    driver.expect("Plant", STATS, fewer)

    add_level(driver, 1)
    # A few nodes of two on each second-level limb, less those that start outside the envelope or
    # would be too short.
    driver.expect("Plant", STATS, {"limbs": Between(9 + 5 + 1, 10 + 40 + 40 * 2 * 4)})
    driver.screenshot("three_levels")

    driver.click("Shell/Panel/ornaments/tip_leaves/off")
    driver.expect("Plant", STATS, {"cards": 0})
    driver.expect("Leaves", VISIBILITY, "Hidden")
    driver.absent("Shell/Panel/ornaments/tip_leaves")
    ornaments = driver.get("Plant", GROWN_FROM, at="ornaments")
    assert "tip_leaves" not in ornaments, ornaments
    # Bare, with every stem drawn at least 10 cm wide: the twigs thicken, the limbs stay.
    driver.set_resource(SPECIES, 0.1, at="detail")
    driver.expect("Plant", GROWN_FROM, Between(0.099, 0.101), at="detail")
    driver.screenshot("thick_detail")

    # Leaves come back, and show, when added again from Foliage's menu.
    driver.click("Shell/Panel/ornaments/add_operator")
    driver.click("Shell/Panel/ornaments/add_operator/add_tip_leaves")
    driver.expect("Plant", STATS, {"cards": Between(20, 1000)})
    driver.expect("Leaves", VISIBILITY, "Inherited")
    driver.screenshot("leaves_again")

    # The second level's ✕ hides the third with it.
    driver.click("Shell/Panel/levels[1]/remove")
    driver.expect("Plant", STATS, {"level_limbs": [first]})
    driver.absent("Shell/Panel/levels[2]")


def cards(driver):
    # A card at each of the nine or ten limbs' tips, and three at the trunk's, eight triangles
    # each.
    leaves = {"leafy_tips": Between(10, 11), "cards": Between(9 + 3, 10 + 3)}
    driver.expect("Plant", STATS, {**leaves, "triangles": Between(150, 2000)})
    driver.screenshot("default_folds")

    droop(driver, lateral=45, axial=0)
    driver.screenshot("drooped")
    droop(driver, lateral=-45, axial=0)
    driver.screenshot("cupped")
    droop(driver, lateral=0, axial=60)
    driver.screenshot("bent")

    # Upright tips, on the trunk and on limbs 10° to 30° from it: a rosette of three each by
    # default, then one card each, then none.
    driver.set_resource(SPECIES, math.radians(20), at="limbs.levels[0].angle.center")
    driver.expect("Plant", STATS, {"leafy_tips": Between(10, 11), "cards": Between(10 * 3, 11 * 3)})
    driver.screenshot("upright_rosettes")
    driver.set_resource(SPECIES, "Single", at=f"{TIP_LEAVES}.upright")
    driver.expect("Plant", STATS, {"leafy_tips": Between(10, 11), "cards": Between(10, 11)})
    driver.screenshot("upright_single")
    driver.set_resource(SPECIES, "Bare", at=f"{TIP_LEAVES}.upright")
    driver.expect("Plant", STATS, {"leafy_tips": 0, "cards": 0})
    driver.screenshot("upright_bare")


def droop(driver, lateral, axial):
    """Sets the cards' droops, in degrees, and waits for the plant grown with them."""
    driver.set_resource(SPECIES, math.radians(lateral), at=f"{TIP_LEAVES}.lateral_droop")
    driver.set_resource(SPECIES, math.radians(axial), at=f"{TIP_LEAVES}.axial_droop")
    axial_radians = math.radians(axial)
    at = "ornaments.tip_leaves.axial_droop"
    driver.expect("Plant", GROWN_FROM, Between(axial_radians - 0.001, axial_radians + 0.001), at=at)
