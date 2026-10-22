"""The reference species in `reference/` and its subfolders, each opened and shot, for the person reading the run to
compare after a change to the layout. A file opens named after itself with no dot, so nothing in
it was pruned."""

from pathlib import Path

BIN = "arbor_day"
REQUEST = "arbor_day::document::FileRequest"
TEXT = "bevy_ui::widget::text::Text"
NAME = "Shell/Panel/document/name"

REFERENCE = Path(__file__).resolve().parents[2] / "reference"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/levels[0]/add")  # the immediate-mode panel has built its rows
    for path in sorted(REFERENCE.rglob("*.gltf")):
        driver.message(REQUEST, {"Open": str(path)})
        driver.expect(NAME, TEXT, path.stem)
        driver.frames(2)
        driver.screenshot(path.stem)
    driver.exit()
