"""The curve editor example: each row's editor beside its curve as its app stores it.
- implicit slopes, stored as `[x, y]` pairs: a point typed into shows in the pairs."""

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
    driver.exit()
