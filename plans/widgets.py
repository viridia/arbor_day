"""The property editor's widgets beyond the plain number:
- a fuzzy number's row: center, spread and a shape toggle whose icon shows the current shape, and
  the spread strip in the center input, a tick where there is no spread;
- an enum's select: picking a profile redraws the crown envelope, drawn as lines around the
  plant, and picking a limb level's facing sets it;
- a color field's row: a swatch that opens a picker, where a typed color tints the trunk."""

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"
ENVELOPE = "arbor_day::plant::EnvelopeStats"
GROWN_FROM = "arbor_day::plant::GrownFrom"
LIMBS = "Shell/Panel/limbs[0]"
COLOR = "Shell/Panel/wood/color/input"


def pick(driver, field, variant):
    """Opens a field's select and picks a variant."""
    driver.click(f"Shell/Panel/{field}/select")
    driver.click(f"Shell/Panel/{field}/{variant}")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present(f"{LIMBS}/angle/shape")  # the immediate-mode panel has built its rows

    # Fuzzy numbers: the first limb level's angle and reach, both bells.
    driver.expect("Plant", GROWN_FROM, "Bell", at="limbs[0].angle.shape")
    driver.expect("Plant", GROWN_FROM, "Bell", at="limbs[0].reach.shape")
    driver.present(f"{LIMBS}/angle/center/spread_strip/segment0")
    driver.absent(f"{LIMBS}/angle/center/spread_strip/segment1")
    driver.screenshot("fuzzy")
    driver.click(f"{LIMBS}/angle/shape")
    driver.expect("Plant", GROWN_FROM, "Uniform", at="limbs[0].angle.shape")
    driver.screenshot("fuzzy_toggled")
    # With no spread, the strip is a tick at the center. A click places the cursor rather than
    # selecting, so clear the default (10%) first.
    driver.click(f"{LIMBS}/reach/spread")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("0")
    driver.key("Enter")
    driver.expect("Plant", GROWN_FROM, 0.0, at="limbs[0].reach.spread")
    driver.present(f"{LIMBS}/reach/center/spread_strip/segment0")
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
    # Any enum of unit variants gets a select, here one in a limb level.
    driver.expect("Plant", GROWN_FROM, "Around", at="limbs[0].facing")
    pick(driver, "limbs[0]/facing", "Up")
    driver.expect("Plant", GROWN_FROM, "Up", at="limbs[0].facing")
    driver.screenshot("picked_variant")

    # The bark's color, under Wood.
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
