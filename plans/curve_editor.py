"""The curve editor example: each row's editor beside its curve as its app stores it.
- implicit slopes, stored as `[x, y]` pairs: with no point selected a placeholder stands in for
  the inputs, and a point typed into shows in the pairs.
- one cubic, stored as CSS's `cubic-bezier`: a selected point shows its handle, and a click on
  the handle leaves the curve and the selection as they were; an offset typed into a handle's
  input shows in the stored form. Drags are checked by hand.
- mixed segments and handle modes, stored whole: a kind and a mode picked from the header's
  dropdowns, for the selected point, show in the stored form, its span drawn bright."""

EXAMPLE = "curve_editor"
TEXT = "bevy_ui::widget::text::Text"
NODE = "bevy_ui::ui_node::Node"


def retype(driver, path, text):
    """Replaces a number input's text. A click places the cursor, so clear the field first."""
    driver.click(path)
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type(text)
    driver.key("Enter")


def run(driver):
    driver.present("auto/editor/plot/point2")
    driver.expect("auto/stored", TEXT, "[\n  [0, 0.8],\n  [0.4, 0.3],\n  [1, 0.6],\n]")
    driver.expect("auto/editor/no_point", NODE, "Flex", at="display")
    driver.screenshot("rows")
    driver.click("auto/editor/plot/point1")
    driver.expect("auto/editor/no_point", NODE, "None", at="display")
    driver.screenshot("auto_selected")
    retype(driver, "auto/editor/y", "0.5")
    driver.expect("auto/stored", TEXT, "[\n  [0, 0.8],\n  [0.4, 0.5],\n  [1, 0.6],\n]")
    driver.screenshot("auto_typed")

    ease = "cubic-bezier(0.25, 0.1, 0.25, 1)"
    driver.expect("bezier/stored", TEXT, ease)
    driver.absent("bezier/editor/plot/out0")
    driver.click("bezier/editor/plot/point0")
    driver.present("bezier/editor/plot/out0")
    driver.present("bezier/editor/plot/stem0")
    driver.absent("bezier/editor/plot/in1")
    driver.click("bezier/editor/plot/out0")
    driver.present("bezier/editor/plot/out0")
    driver.expect("bezier/stored", TEXT, ease)
    driver.screenshot("bezier_handle")
    retype(driver, "bezier/editor/out_dy", "0.3")
    driver.expect("bezier/stored", TEXT, "cubic-bezier(0.25, 0.3, 0.25, 1)")
    driver.click("bezier/editor/plot/point1")
    driver.present("bezier/editor/plot/in1")
    driver.absent("bezier/editor/plot/out0")
    retype(driver, "bezier/editor/in_dx", "-0.5")
    driver.expect("bezier/stored", TEXT, "cubic-bezier(0.25, 0.3, 0.5, 1)")
    driver.screenshot("bezier_other_handle")

    point = "CurvePoint { x: 0.3, y: 0.55, segment: "
    assert point + "Cubic, handles: Aligned" in driver.get("mixed/stored", TEXT)
    driver.absent("mixed/editor/plot/span1")
    driver.click("mixed/editor/plot/point1")
    driver.present("mixed/editor/plot/span1")
    driver.click("mixed/kind")
    driver.screenshot("mixed_kind_menu")
    driver.click("mixed/kind/linear")
    driver.expect("mixed/stored", TEXT, Contains(point + "Linear, handles: Aligned"))
    driver.click("mixed/mode")
    driver.click("mixed/mode/free")
    driver.expect("mixed/stored", TEXT, Contains(point + "Linear, handles: Free"))
    driver.screenshot("mixed_picked")
    driver.exit()


class Contains:
    """Matches text holding `part`, for `expect`."""

    def __init__(self, part):
        self.part = part

    def __eq__(self, found):
        return isinstance(found, str) and self.part in found

    def __repr__(self):
        return f"text containing {self.part!r}"
