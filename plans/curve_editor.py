"""The curve editor example: each row's editor beside its curve as its app stores it.
- implicit slopes, stored as `[x, y]` pairs: a point typed into shows in the pairs.
- one cubic, stored as CSS's `cubic-bezier`: a selected point shows its handle, and a click on
  the handle leaves the curve and the selection as they were; an offset typed into a handle's
  input shows in the stored form. Drags are checked by hand."""

EXAMPLE = "curve_editor"
TEXT = "bevy_ui::widget::text::Text"


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
    driver.screenshot("rows")
    driver.click("auto/editor/plot/point1")
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
    driver.exit()
