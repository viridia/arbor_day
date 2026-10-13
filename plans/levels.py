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
    # Ten limbs on each of the first level's eight.
    driver.expect("Plant", STATS, {"limbs": 88})
    driver.absent("Shell/Panel/limbs[0]/add")  # the buttons move to the last level
    driver.screenshot("two_levels")

    # Two nodes of two on each first-level limb.
    driver.click("Shell/Panel/limbs[1]/nodes/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("2")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"limbs": 40})

    driver.click("Shell/Panel/limbs[1]/remove")
    driver.expect("Plant", STATS, {"limbs": 8})
    driver.absent("Shell/Panel/limbs[1]")
    driver.click("Shell/Panel/limbs[0]/add")
    driver.expect("Plant", STATS, {"limbs": 40})

    driver.click("Shell/Panel/limbs[1]/add")
    # Up to six on each of the 32 second-level limbs, less those that start outside the envelope
    # or would be too short.
    driver.expect("Plant", STATS, {"limbs": Between(40 + 1, 40 + 32 * 6)})
    driver.screenshot("three_levels")
    driver.exit()
