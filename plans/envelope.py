"""The crown envelope drawn as lines around the plant, and an enum's select: picking a profile
redraws the envelope, and picking a variant of the placeholder's enum sets it."""

BIN = "arbor_day"
ENVELOPE = "arbor_day::plant::EnvelopeStats"
PLACEHOLDER = "arbor_day::Placeholder"


def pick(driver, field, variant):
    """Opens a field's select and picks a variant."""
    driver.click(f"Shell/Panel/{field}/select")
    driver.click(f"Shell/Panel/{field}/{variant}")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/profile/select")  # the immediate-mode panel has built its rows
    # The default ellipsoid closes at both ends: 7 rings of 24 and 8 meridians of 16.
    driver.expect("Envelope", ENVELOPE, {"segments": 296})
    driver.screenshot("envelope")
    driver.click("Shell/Panel/profile/select")
    driver.screenshot("profile_menu")
    driver.click("Shell/Panel/profile/Cone")
    # A cone is open at its base: 8 rings, and a spoke across the base on each meridian.
    driver.expect("Envelope", ENVELOPE, {"segments": 328})
    driver.screenshot("cone")
    pick(driver, "profile", "Column")
    driver.expect("Envelope", ENVELOPE, {"segments": 360})
    # Any enum of unit variants gets a select, here one below the fold.
    driver.expect("Placeholder", PLACEHOLDER, {"mode": "First"})
    pick(driver, "mode", "Second")
    driver.expect("Placeholder", PLACEHOLDER, {"mode": "Second"})
    driver.screenshot("picked")
    driver.exit()
