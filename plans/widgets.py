"""The property editor's widgets beyond the plain number:
- a fuzzy number's row: center, spread and a shape toggle whose icon shows the current shape, and
  the spread strip in the center input, a tick where there is no spread;
- an interval's row: a fuzzy number's without the shape toggle;
- an enum's select: picking a profile redraws the crown envelope, drawn as lines around the
  plant, and picking a limb level's facing sets it;
- a checkbox: the outline, whose bark hull hides when it is cleared;
- a color field's row: a swatch that opens a picker, where a typed color tints the trunk;
- a curve's row: Bias's Size as a thumbnail, opened into its editor, where a point is selected
  and typed into, a double-click inserts one and Delete removes it, Tab selects the next point and
  arrows move it, and a double-click on a point deletes it; then a step and a steep rise, judged by eye for facets and an even width. The
  driver cannot drag, so dragging a point, selecting by rectangle and Escape are checked by
  hand."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"
ENVELOPE = "arbor_day::plant::EnvelopeStats"
GROWN_FROM = "arbor_day::plant::GrownFrom"
SPECIES = "arbor_day::species::Species"
VISIBILITY = "bevy_camera::visibility::Visibility"
LIMBS = "Shell/Panel/levels[0]"
COLOR = "Shell/Panel/bark_color/input"
SPACING = "Shell/Panel/tip_leaves/sprig/spacing"


def pick(driver, field, variant):
    """Opens a field's select and picks a variant."""
    driver.click(f"Shell/Panel/{field}/select")
    driver.click(f"Shell/Panel/{field}/{variant}")


def retype(driver, path, text):
    """Replaces a number input's text. A click places the cursor, so clear the field first."""
    driver.click(path)
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type(text)
    driver.key("Enter")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present(f"{LIMBS}/reach/shape")  # the immediate-mode panel has built its rows

    # Fuzzy numbers: the first limb level's fork angle and reach, both bells. The fork's, since
    # `levels[0]/angle` would also match the angles nested under it; the fork is off by default.
    driver.set_resource(SPECIES, "On", at="limbs.levels[0].fork.state")
    driver.expect("Plant", GROWN_FROM, "Bell", at="limbs[0].fork.angle.shape")
    driver.expect("Plant", GROWN_FROM, "Bell", at="limbs[0].reach.shape")
    driver.present(f"{LIMBS}/fork/angle/center/spread_strip/segment0")
    driver.absent(f"{LIMBS}/fork/angle/center/spread_strip/segment1")
    driver.screenshot("fuzzy")
    driver.click(f"{LIMBS}/fork/angle/shape")
    driver.expect("Plant", GROWN_FROM, "Uniform", at="limbs[0].fork.angle.shape")
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

    # An interval: a fuzzy row without the shape button, here the painted sprig's spacing, whose
    # default has no spread.
    driver.absent(f"{SPACING}/shape")
    driver.click(f"{SPACING}/spread")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type("10")
    driver.key("Enter")
    driver.expect("Plant", GROWN_FROM, Between(0.099, 0.101), at="ornaments.tip_leaves.sprig.spacing.spread")
    driver.present(f"{SPACING}/center/spread_strip/segment0")
    driver.screenshot("interval")

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
    pick(driver, "levels[0]/facing", "Up")
    driver.expect("Plant", GROWN_FROM, "Up", at="limbs[0].facing")
    driver.screenshot("picked_variant")

    # A checkbox: the outline, on by default, drawn around the bark and painted around the leaves.
    driver.expect("Outline", VISIBILITY, "Inherited")
    driver.screenshot("outlined")
    driver.click("Shell/Panel/outline")
    driver.expect("Plant", GROWN_FROM, False, at="outline")
    driver.expect("Outline", VISIBILITY, "Hidden")
    driver.screenshot("not_outlined")

    # The bark's color, under Plant.
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

    # A curve: the first level's Bias Size, flat at 100% by default. A click lands on the plot
    # under a point's marker, which selects that point.
    driver.set_resource(SPECIES, "On", at="limbs.levels[0].bias.state")
    size = f"{LIMBS}/bias/size"
    driver.present(f"{size}/thumbnail")
    driver.click(f"{size}/open")
    driver.absent(f"{size}/thumbnail")
    driver.click(f"{size}/editor/plot/point1")
    retype(driver, f"{size}/editor/y", "40")
    driver.expect("Plant", GROWN_FROM, Between(0.39, 0.41), at="limbs[0].bias.size[1][1]")
    driver.screenshot("curve_typed")
    # A double-click away from the points inserts one on the curve, at the middle; Delete, with
    # the plot focused, removes it again.
    driver.click(f"{size}/editor/plot")
    driver.click(f"{size}/editor/plot")
    driver.expect("Plant", GROWN_FROM, Between(0.49, 0.51), at="limbs[0].bias.size[1][0]")
    driver.present(f"{size}/editor/plot/point2")
    driver.screenshot("curve_inserted")
    driver.key("Delete")
    driver.absent(f"{size}/editor/plot/point2")
    driver.expect("Plant", GROWN_FROM, Between(0.99, 1.01), at="limbs[0].bias.size[1][0]")
    # Keys, the plot still focused and nothing selected: Tab selects the base, an arrow moves it
    # a step of 1%, ten with Shift; Tab again selects the tip.
    driver.key("Tab")
    driver.key("ArrowDown")
    driver.expect("Plant", GROWN_FROM, Between(0.985, 0.995), at="limbs[0].bias.size[0][1]")
    driver.press("ShiftLeft")
    driver.key("ArrowUp")
    driver.release("ShiftLeft")
    driver.expect("Plant", GROWN_FROM, Between(1.085, 1.095), at="limbs[0].bias.size[0][1]")
    driver.key("Tab")
    driver.key("ArrowDown")
    driver.expect("Plant", GROWN_FROM, Between(0.385, 0.395), at="limbs[0].bias.size[1][1]")
    driver.expect("Plant", GROWN_FROM, Between(1.085, 1.095), at="limbs[0].bias.size[0][1]")
    # A double-click on a point deletes it: one inserted at the middle again, then removed.
    driver.click(f"{size}/editor/plot")
    driver.click(f"{size}/editor/plot")
    driver.present(f"{size}/editor/plot/point2")
    driver.click(f"{size}/editor/plot/point1")
    driver.click(f"{size}/editor/plot/point1")
    driver.absent(f"{size}/editor/plot/point2")
    driver.expect("Plant", GROWN_FROM, Between(0.99, 1.01), at="limbs[0].bias.size[1][0]")
    # A step at the middle and a steep rise after it.
    shape = [[0.0, 0.4], [0.5, 0.4], [0.5, 1.6], [0.56, 2.0], [1.0, 0.2]]
    driver.set_resource(SPECIES, shape, at="limbs.levels[0].bias.settings.size.0")
    driver.present(f"{size}/editor/plot/point4")
    driver.screenshot("curve_shapes")
    # Closed again, the row shows the same curve as a thumbnail.
    driver.click(f"{size}/open")
    driver.present(f"{size}/thumbnail")
    driver.absent(f"{size}/editor")
    driver.screenshot("curve_closed")
    driver.exit()
