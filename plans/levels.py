"""Limb levels: + shows the next level, growing on the limbs of the one before, and − hides the
last. A hidden level keeps its parameters, so showing it again brings it back as it was. Leaf cards
grow at branch ends and in the crown's shell, and no foliage density leaves the tree bare."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"
GROWN_FROM = "arbor_day::plant::GrownFrom"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/limbs[0]/add")  # the immediate-mode panel has built its rows
    # Nine or ten, and the one carrying the trunk on.
    first = Between(10, 11)
    driver.expect(
        "Plant", STATS, {"limbs": first, "level_limbs": [first], "cards": Between(20, 150)}
    )
    driver.click("Shell/Panel/limbs[0]/add")
    # One to six nodes of one or two on each of the first level's ten or eleven limbs, 1 to 3 m
    # long, and one carrying each on. Each level's count shows in its header.
    second = Between(10 * 2, 11 * (2 * 6 + 1))
    driver.expect("Plant", STATS, {"level_limbs": [first, second]})
    driver.absent("Shell/Panel/limbs[0]/add")  # the buttons move to the last level
    driver.screenshot("two_levels")

    # Fewer nodes: up to two on each first-level limb, and some bearing none.
    driver.click("Shell/Panel/limbs[1]/density/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("1.5")
    driver.key("Enter")
    fewer = {"level_limbs": [first, Between(5, 11 * (2 * 2 + 1))]}
    driver.expect("Plant", STATS, fewer)

    driver.click("Shell/Panel/limbs[1]/remove")
    driver.expect("Plant", STATS, {"level_limbs": [first]})
    driver.absent("Shell/Panel/limbs[1]")
    driver.click("Shell/Panel/limbs[0]/add")
    driver.expect("Plant", STATS, fewer)

    driver.click("Shell/Panel/limbs[1]/add")
    # A few nodes of two on each second-level limb, less those that start outside the envelope or
    # would be too short, and one carrying each on.
    driver.expect("Plant", STATS, {"limbs": Between(10 + 5 + 1, 11 + 55 + 55 * (2 * 4 + 1))})
    driver.screenshot("three_levels")

    driver.click("Shell/Panel/foliage/density/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("0")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"cards": 0})
    # Bare, with every stem drawn at least 10 cm wide: the twigs thicken, the limbs stay.
    driver.click("Shell/Panel/bark/detail/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("0.1")
    driver.key("Enter")
    driver.expect("Plant", GROWN_FROM, Between(0.099, 0.101), at="bark.detail")
    driver.screenshot("thick_detail")
    driver.exit()
