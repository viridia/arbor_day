"""Node bias: the first level's nodes crowd toward the trunk's tip, then its base. The density
alone sets how many, so the count holds; compare the screenshots for where they sit."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"


def set_bias(driver, text):
    """Replaces the first level's bias. A click places the cursor rather than selecting, so clear
    what is there first."""
    driver.click("Shell/Panel/limbs[0]/bias/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type(text)
    driver.key("Enter")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/limbs[0]/bias/input")  # the immediate-mode panel has built its rows
    count = Between(9, 10)
    driver.expect("Plant", STATS, {"limbs": count})
    driver.screenshot("even")
    set_bias(driver, "100")
    driver.expect("Plant", STATS, {"limbs": count})
    driver.screenshot("toward_tip")
    set_bias(driver, "-100")
    driver.expect("Plant", STATS, {"limbs": count})
    driver.screenshot("toward_base")
    driver.exit()
