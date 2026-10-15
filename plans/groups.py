"""Control groups: the seed and the detail threshold share a Plant group at the top of the panel,
and each limb level's rows sit in Arrangement, Direction and Shape groups. Checked by eye in the
screenshots."""

BIN = "arbor_day"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/limbs[0]/add")  # the immediate-mode panel has built its rows
    driver.present("Shell/Panel/detail/input")
    driver.screenshot("plant_group")
    # Focusing an input scrolls the panel to it.
    driver.click("Shell/Panel/limbs[0]/skew/input")
    driver.key("Escape")
    driver.screenshot("limb_groups")
    driver.exit()
