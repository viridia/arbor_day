"""Foliage: the default tree carries leaf cards in the crown's shell, a second level moves them
onto its twigs, and no density leaves the tree bare. Compare the screenshots for the cards."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/foliage/density/input")  # the immediate-mode panel has built its rows
    driver.expect("Plant", STATS, {"cards": Between(20, 150)})
    driver.screenshot("default")

    driver.click("Shell/Panel/limbs[0]/add")
    driver.expect("Plant", STATS, {"level_limbs": [Between(9, 10), Between(1, 400)]})
    driver.screenshot("twigs")

    # A click places the cursor rather than selecting, so clear what is there first.
    driver.click("Shell/Panel/foliage/density/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("0")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"cards": 0})
    driver.screenshot("bare")
    driver.exit()
