"""Facing and skew: the second level's limbs on the first level's, to the sides (the default), up,
down and around, then skewed. Every facing keeps the count; compare the screenshots for which way
the limbs leave."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"


def pick(driver, variant):
    """Opens the second level's facing select and picks a variant."""
    driver.click("Shell/Panel/limbs[1]/facing/select")
    driver.click(f"Shell/Panel/limbs[1]/facing/{variant}")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/limbs[0]/add")  # the immediate-mode panel has built its rows
    driver.click("Shell/Panel/limbs[0]/add")
    two_levels = {"level_limbs": [Between(9, 10), Between(1, 400)]}
    driver.expect("Plant", STATS, two_levels)
    driver.screenshot("sides")
    for variant in ["Up", "Down", "Around"]:
        pick(driver, variant)
        driver.expect("Plant", STATS, two_levels)
        driver.screenshot(variant.lower())

    # The most skew turns each limb up to 90° from where its facing put it.
    driver.click("Shell/Panel/limbs[1]/skew/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("90")
    driver.key("Enter")
    driver.expect("Plant", STATS, two_levels)
    driver.screenshot("skewed")
    driver.exit()
