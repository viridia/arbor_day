"""Card folds: the lateral droop turns each half of a card down about its midrib, or up, cupping
it, when negative; the axial droop bends its far half down. Checked by value, and by eye in the
screenshots."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"
GROWN_FROM = "arbor_day::plant::GrownFrom"
SPECIES = "arbor_day::species::Species"


def set_number(driver, path, text):
    """Replaces a number input's text. A click places the cursor rather than selecting, so clear
    what is there first."""
    driver.click(f"Shell/Panel/{path}/input")
    driver.key("End")
    for _ in range(8):
        driver.key("Backspace")
    driver.type(text)
    driver.key("Enter")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/levels[0]/add")  # the immediate-mode panel has built its rows
    # A card at each of the ten or eleven limbs' tips, or up to three where it is steep, eight
    # triangles each.
    leaves = {"leafy_tips": Between(10, 11), "cards": Between(10, 33)}
    driver.expect("Plant", STATS, {**leaves, "triangles": Between(300, 2000)})
    driver.screenshot("default_folds")

    set_number(driver, "tip_leaves/lateral_droop", "45")
    driver.expect("Plant", GROWN_FROM, Between(0.785, 0.786), at="ornaments.tip_leaves.lateral_droop")
    set_number(driver, "tip_leaves/axial_droop", "0")
    driver.expect("Plant", GROWN_FROM, 0.0, at="ornaments.tip_leaves.axial_droop")
    driver.screenshot("drooped")

    set_number(driver, "tip_leaves/lateral_droop", "-45")
    driver.expect("Plant", GROWN_FROM, Between(-0.786, -0.785), at="ornaments.tip_leaves.lateral_droop")
    driver.screenshot("cupped")

    set_number(driver, "tip_leaves/lateral_droop", "0")
    set_number(driver, "tip_leaves/axial_droop", "60")
    driver.expect("Plant", GROWN_FROM, Between(1.047, 1.048), at="ornaments.tip_leaves.axial_droop")
    driver.screenshot("bent")

    # Upright tips: one card each, then none on the steep ones, which the default tree has.
    driver.set_resource(SPECIES, "Single", at="ornaments.tip_leaves.upright")
    driver.expect("Plant", STATS, {"leafy_tips": Between(10, 11), "cards": Between(10, 11)})
    driver.screenshot("upright_single")
    driver.set_resource(SPECIES, "Bare", at="ornaments.tip_leaves.upright")
    driver.expect("Plant", STATS, {"leafy_tips": Between(1, 10), "cards": Between(1, 10)})
    driver.screenshot("upright_bare")
    driver.exit()
