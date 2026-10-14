"""Limbs bend and kink as they grow, and the seed sets the kinks: a typed seed reaches the plant,
and Random picks another."""

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/seed/input")  # the immediate-mode panel has built its rows
    driver.expect("Plant", STATS, {"seed": 1})
    driver.click("Shell/Panel/limbs[0]/add")
    driver.click("Shell/Panel/limbs[1]/add")
    driver.screenshot("seed_1")

    # A click places the cursor rather than selecting, so clear what is there first.
    driver.click("Shell/Panel/seed/input")
    driver.key("End")
    for _ in range(8):
        driver.key("Backspace")
    driver.type("42")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"seed": 42})
    driver.screenshot("seed_42")

    driver.click("Shell/Panel/seed/randomize")
    driver.frames(2)
    driver.screenshot("random_seed")
    driver.exit()
