"""Limb levels: the last level's add menu shows the next, growing on the limbs of the one before,
and a level's ✕ hides it and those after it. A hidden level keeps its parameters, so showing it
again brings it back as it was. A leaf card grows at each branch end, and the tip leaves' ✕ leaves
the tree bare until Foliage's add menu brings them back."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"
GROWN_FROM = "arbor_day::plant::GrownFrom"
VISIBILITY = "bevy_camera::visibility::Visibility"


def add_level(driver, last):
    """Shows the level after `last`, the last shown, from its header's add menu."""
    driver.click(f"Shell/Panel/levels[{last}]/add_operator")
    driver.click(f"Shell/Panel/levels[{last}]/add_operator/add")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/levels[0]/add")  # the immediate-mode panel has built its rows
    # Nine or ten, and the one carrying the trunk on, each bearing a card at its tip, or up to
    # three where it is steep.
    first = Between(10, 11)
    leaves = {"leafy_tips": first, "cards": Between(10, 33)}
    driver.expect("Plant", STATS, {"limbs": first, "level_limbs": [first], **leaves})
    add_level(driver, 0)
    # One to six nodes of one or two on each of the first level's ten or eleven limbs, 1 to 3 m
    # long, and one carrying each on. Each level's count shows in its header.
    second = Between(10 * 2, 11 * (2 * 6 + 1))
    driver.expect("Plant", STATS, {"level_limbs": [first, second]})
    driver.absent("Shell/Panel/levels[0]/add")  # the next level is offered by the last only
    driver.absent("Shell/Panel/levels[0]/remove")  # the first level stays
    driver.screenshot("two_levels")

    # Fewer nodes: up to two on each first-level limb, and some bearing none.
    driver.click("Shell/Panel/levels[1]/density/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("1.5")
    driver.key("Enter")
    fewer = {"level_limbs": [first, Between(5, 11 * (2 * 2 + 1))]}
    driver.expect("Plant", STATS, fewer)

    driver.click("Shell/Panel/levels[1]/remove")
    driver.expect("Plant", STATS, {"level_limbs": [first]})
    driver.absent("Shell/Panel/levels[1]")
    add_level(driver, 0)
    driver.expect("Plant", STATS, fewer)

    add_level(driver, 1)
    # A few nodes of two on each second-level limb, less those that start outside the envelope or
    # would be too short, and one carrying each on.
    driver.expect("Plant", STATS, {"limbs": Between(10 + 5 + 1, 11 + 55 + 55 * (2 * 4 + 1))})
    driver.screenshot("three_levels")

    driver.click("Shell/Panel/ornaments/tip_leaves/off")
    driver.expect("Plant", STATS, {"cards": 0})
    driver.expect("Leaves", VISIBILITY, "Hidden")
    driver.absent("Shell/Panel/ornaments/tip_leaves")
    ornaments = driver.get("Plant", GROWN_FROM, at="ornaments")
    assert "tip_leaves" not in ornaments, ornaments
    # Bare, with every stem drawn at least 10 cm wide: the twigs thicken, the limbs stay.
    driver.click("Shell/Panel/detail/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("0.1")
    driver.key("Enter")
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
    driver.exit()
