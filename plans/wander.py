"""Wander as kinks: the first level's limbs zigzag about their course at the most wander, 45°,
rather than curling away, and the second level's twigs kink finer. The count holds; compare the
screenshots for the shapes."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/limbs[0]/wander/input")  # the immediate-mode panel has built its rows
    first = Between(9, 10)
    driver.expect("Plant", STATS, {"limbs": first})
    driver.screenshot("default")

    # A click places the cursor rather than selecting, so clear what is there first.
    driver.click("Shell/Panel/limbs[0]/wander/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("45")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"limbs": first})
    driver.screenshot("most_wander")

    driver.click("Shell/Panel/limbs[0]/add")
    driver.expect("Plant", STATS, {"level_limbs": [first, Between(1, 400)]})
    driver.screenshot("twigs")
    driver.exit()
