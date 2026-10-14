"""Limb levels: + shows the next level, growing on the limbs of the one before, and − hides the
last. A hidden level keeps its parameters, so showing it again brings it back as it was."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/limbs[0]/add")  # the immediate-mode panel has built its rows
    first = Between(9, 10)
    driver.expect("Plant", STATS, {"limbs": first, "level_limbs": [first]})
    driver.click("Shell/Panel/limbs[0]/add")
    # Nodes of two on each of the first level's nine or ten, about three and a half per metre of
    # limbs 1 to 3 m long. Each level's count shows in its header.
    driver.expect("Plant", STATS, {"level_limbs": [first, Between(9 * 2 * 2, 10 * 2 * 12)]})
    driver.absent("Shell/Panel/limbs[0]/add")  # the buttons move to the last level
    driver.screenshot("two_levels")

    # Fewer nodes: one to five of two on each first-level limb.
    driver.click("Shell/Panel/limbs[1]/density/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("4")
    driver.key("Enter")
    fewer = {"level_limbs": [first, Between(9 * 2, 10 * 2 * 5)]}
    driver.expect("Plant", STATS, fewer)

    driver.click("Shell/Panel/limbs[1]/remove")
    driver.expect("Plant", STATS, {"level_limbs": [first]})
    driver.absent("Shell/Panel/limbs[1]")
    driver.click("Shell/Panel/limbs[0]/add")
    driver.expect("Plant", STATS, fewer)

    driver.click("Shell/Panel/limbs[1]/add")
    # Up to a dozen nodes of two on each second-level limb, less those that start outside the
    # envelope or would be too short.
    driver.expect("Plant", STATS, {"limbs": Between(9 + 9 * 2 + 1, 10 + 100 + 100 * 2 * 12)})
    driver.screenshot("three_levels")

    # Limbs under the thinnest meshed radius are still grown but left to the foliage: at 10 cm,
    # only the trunk is meshed. Bare, so the triangles are the bark's.
    driver.click("Shell/Panel/foliage/density/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("0")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"cards": 0})
    driver.click("Shell/Panel/bark/min_radius/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("0.1")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"triangles": Between(1, 100)})
    driver.screenshot("trunk_only")
    driver.exit()
