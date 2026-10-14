"""The property editor's widgets beyond the plain number:
- a fuzzy number's row: center, spread and a shape toggle whose icon shows the current shape, and
  the spread strip in the center input, two segments where a wrapping spread crosses the seam and
  a tick where there is no spread;
- an enum's select: picking a profile redraws the crown envelope, drawn as lines around the
  plant, and picking a variant of the placeholder's enum sets it;
- a color field's row: a swatch that opens a picker, where a typed color tints the trunk."""

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"
ENVELOPE = "arbor_day::plant::EnvelopeStats"
PLACEHOLDER = "arbor_day::Placeholder"
COLOR = "Shell/Panel/bark/color/input"


def pick(driver, field, variant):
    """Opens a field's select and picks a variant."""
    driver.click(f"Shell/Panel/{field}/select")
    driver.click(f"Shell/Panel/{field}/{variant}")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/tilt/shape")  # the immediate-mode panel has built its rows

    # Fuzzy numbers.
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

    # Selects. The default ellipsoid closes at both ends: 7 rings of 24 and 8 meridians of 16.
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
    driver.screenshot("picked_variant")

    # The bark's color.
    driver.expect("Plant", STATS, {"bark_color": "#6B5440"})
    driver.click(COLOR)
    driver.present(f"{COLOR}/wheel")
    driver.screenshot("picker")
    # The hex field shows in RGB mode. A click places the cursor, so clear the field first.
    driver.click(f"{COLOR}/mode_rgb")
    driver.click(f"{COLOR}/hex_input")
    driver.key("End")
    for _ in range(9):
        driver.key("Backspace")
    driver.type("336699")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"bark_color": "#336699"})
    driver.screenshot("picked_color")
    driver.key("Escape")
    driver.absent(f"{COLOR}/wheel")
    driver.screenshot("closed")
    driver.exit()
