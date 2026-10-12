"""A fuzzy number's row: center, spread and a shape toggle whose icon shows the current shape."""

BIN = "arbor_day"
PLACEHOLDER = "arbor_day::Placeholder"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/tilt/shape")  # the immediate-mode panel has built its rows
    driver.expect("Placeholder", PLACEHOLDER, {"length": {"shape": "Uniform"}})
    driver.expect("Placeholder", PLACEHOLDER, {"tilt": {"shape": "Bell"}})
    driver.screenshot("fuzzy")
    driver.click("Shell/Panel/tilt/shape")
    driver.expect("Placeholder", PLACEHOLDER, {"tilt": {"shape": "Uniform"}})
    driver.click("Shell/Panel/length/shape")
    driver.expect("Placeholder", PLACEHOLDER, {"length": {"shape": "Bell"}})
    driver.screenshot("fuzzy_toggled")
    driver.exit()
