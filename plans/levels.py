"""Limb levels: + shows the next level, growing on the limbs of the one before, and − hides the
last. A hidden level keeps its parameters, so showing it again brings it back as it was."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/limbs[0]/add")  # the immediate-mode panel has built its rows
    driver.expect("Plant", STATS, {"limbs": 8})
    driver.click("Shell/Panel/limbs[0]/add")
    # Nodes of two on each of the first level's eight, about two per metre of a 1 to 3 m span.
    driver.expect("Plant", STATS, {"limbs": Between(8 + 8 * 2 * 2, 8 + 8 * 2 * 7)})
    driver.absent("Shell/Panel/limbs[0]/add")  # the buttons move to the last level
    driver.screenshot("two_levels")

    # Fewer nodes: one to three of two on each first-level limb.
    driver.click("Shell/Panel/limbs[1]/density/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("4")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"limbs": Between(8 + 8 * 2, 8 + 8 * 2 * 3)})

    driver.click("Shell/Panel/limbs[1]/remove")
    driver.expect("Plant", STATS, {"limbs": 8})
    driver.absent("Shell/Panel/limbs[1]")
    driver.click("Shell/Panel/limbs[0]/add")
    driver.expect("Plant", STATS, {"limbs": Between(8 + 8 * 2, 8 + 8 * 2 * 3)})

    driver.click("Shell/Panel/limbs[1]/add")
    # About three nodes of two on each second-level limb, less those that start outside the
    # envelope or would be too short.
    driver.expect("Plant", STATS, {"limbs": Between(8 + 8 * 2 + 1, 8 + 48 + 48 * 2 * 6)})
    driver.screenshot("three_levels")
    driver.exit()
