"""A fuzzy number's row: center, spread and a shape toggle whose icon shows the current shape, and
the spread strip in the center input, two segments where a wrapping spread crosses the seam and a
tick where there is no spread."""

BIN = "arbor_day"
PLACEHOLDER = "arbor_day::Placeholder"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/tilt/shape")  # the immediate-mode panel has built its rows
    driver.expect("Placeholder", PLACEHOLDER, {"length": {"shape": "Uniform"}})
    driver.expect("Placeholder", PLACEHOLDER, {"tilt": {"shape": "Bell"}})
    driver.present("Shell/Panel/length/center/spread_strip/segment0")
    driver.absent("Shell/Panel/length/center/spread_strip/segment1")
    driver.present("Shell/Panel/axis/center/spread_strip/segment1")
    driver.screenshot("fuzzy")
    driver.click("Shell/Panel/tilt/shape")
    driver.expect("Placeholder", PLACEHOLDER, {"tilt": {"shape": "Uniform"}})
    driver.click("Shell/Panel/length/shape")
    driver.expect("Placeholder", PLACEHOLDER, {"length": {"shape": "Bell"}})
    driver.screenshot("fuzzy_toggled")
    # With no spread, the strip is a tick at the center. Clear the default (0.25) first.
    driver.click("Shell/Panel/length/spread")
    driver.key("End")
    for _ in range(4):
        driver.key("Backspace")
    driver.type("0")
    driver.key("Enter")
    driver.expect("Placeholder", PLACEHOLDER, {"length": {"spread": 0.0}})
    driver.present("Shell/Panel/length/center/spread_strip/segment0")
    driver.screenshot("no_spread")
    driver.exit()
