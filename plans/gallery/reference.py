"""The reference species in `reference/` and its subfolders, each opened, framed and shot from the
front and from above, for the person reading the run to compare after a change to the layout. A
file opens named after itself with no dot, so nothing in it was pruned."""

from pathlib import Path

BIN = "arbor_day"
REQUEST = "arbor_day::document::FileRequest"
TEXT = "bevy_ui::widget::text::Text"
TURNTABLE = "arbor_day::turntable::TurntableCamera"
NAME = "Shell/Panel/document/name"

REFERENCE = Path(__file__).resolve().parents[2] / "reference"

# Near the turntable's highest elevation, looking down on the crown.
ABOVE = 1.45


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/levels[0]/add")  # the immediate-mode panel has built its rows
    # The scale stands beside the plant, so framing with it shown leaves a small plant small.
    driver.click("Shell/View/scale")
    for path in sorted(REFERENCE.rglob("*.gltf")):
        driver.message(REQUEST, {"Open": str(path)})
        driver.expect(NAME, TEXT, path.stem)
        driver.click("Shell/View/frame")
        driver.expect("SceneCamera", TURNTABLE, {"frame_pending": False})
        driver.frames(2)
        driver.screenshot(path.stem)
        driver.set("SceneCamera", TURNTABLE, ABOVE, at="elevation")
        driver.frames(2)
        driver.screenshot(f"{path.stem}-above")
    driver.exit()
