"""The panel's structure and the file commands.

Control groups: the seed and the detail threshold share a Plant group at the top of the panel,
and each limb level's rows sit in Arrangement, Direction and Shape groups. Checked by eye in the
screenshots. Scrolled down to the limb groups, the header and its file menu stay in place.

Operators: a level's Wander starts off, and its header's add menu lists it and turns it on. The
✕ turns it off again, hiding its rows and leaving it out of the species as the file writes it, and
the menu brings it back with the value it had. The eye beside the ✕ disables it, its rows kept and
the file marking it, and enables it again. The trunk's Wander has the same menu and ✕.

File commands, from a new document: the header row names it "Untitled" and its file menu lists
New, Open, Save and Save As. Saved as (a `FileRequest`, since the driver cannot work native
dialogs), the row names the document after its file and the glTF, its buffer and leaf texture are
written; an edit marks the document with a dot and Cmd+S clears it. New returns to the default
species, untitled, and Open brings the saved species back. A file holding a key the species lacks
and a value of the wrong type opens with the rest, marked with the dot."""

import json
import os
import tempfile

from bevy_remote_driver import Between

BIN = "arbor_day"
GROWN_FROM = "arbor_day::plant::GrownFrom"
SPECIES = "arbor_day::species::Species"
REQUEST = "arbor_day::document::FileRequest"
TEXT = "bevy_ui::widget::text::Text"
NAME = "Shell/Panel/document/name"
MENU = "Shell/Panel/document/file_menu"


def run(driver):
    driver.present("Shell")  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/levels[0]/add")  # the immediate-mode panel has built its rows
    groups(driver)
    operators(driver)
    new(driver)
    files(driver)
    driver.exit()


def new(driver):
    driver.click(MENU)
    driver.click(f"{MENU}/new")


def groups(driver):
    driver.present("Shell/Panel/detail/input")
    driver.screenshot("plant_group")
    # Focusing an input scrolls the panel to it.
    driver.click("Shell/Panel/levels[0]/skew/input")
    driver.key("Escape")
    driver.screenshot("limb_groups")
    # The header stays above the rows that scroll.
    driver.click(MENU)
    driver.present(f"{MENU}/save")
    driver.key("Escape")


def operators(driver):
    driver.present("Shell/Panel/levels[0]/add_operator")
    driver.absent("Shell/Panel/levels[0]/wander")  # off by default
    level = driver.get("Plant", GROWN_FROM, at="limbs[0]")
    assert "wander" not in level, level

    driver.click("Shell/Panel/levels[0]/add_operator")
    driver.present("Shell/Panel/levels[0]/add_operator/add_fork")
    driver.screenshot("add_menu")
    driver.click("Shell/Panel/levels[0]/add_operator/add_wander")
    driver.present("Shell/Panel/levels[0]/wander/angle")
    driver.set_resource(SPECIES, 0.5, at="limbs.levels[0].wander.settings.angle")
    driver.expect("Plant", GROWN_FROM, Between(0.499, 0.501), at="limbs[0].wander.angle")
    driver.frames(2)  # the new rows laid out before the ✕ below the fold is clicked (ledger C93)

    driver.click("Shell/Panel/levels[0]/wander/off")
    driver.absent("Shell/Panel/levels[0]/wander")
    level = driver.get("Plant", GROWN_FROM, at="limbs[0]")
    assert "wander" not in level, level
    driver.screenshot("wander_off")
    driver.click("Shell/Panel/levels[0]/add_operator")
    driver.click("Shell/Panel/levels[0]/add_operator/add_wander")
    driver.expect("Plant", GROWN_FROM, Between(0.499, 0.501), at="limbs[0].wander.angle")

    driver.click("Shell/Panel/levels[0]/wander/enabled")
    driver.expect("Plant", GROWN_FROM, False, at="limbs[0].wander.enabled")
    driver.present("Shell/Panel/levels[0]/wander/angle")
    driver.screenshot("wander_disabled")
    driver.click("Shell/Panel/levels[0]/wander/enabled")
    driver.expect("Plant", GROWN_FROM, Between(0.499, 0.501), at="limbs[0].wander.angle")
    wander = driver.get("Plant", GROWN_FROM, at="limbs[0].wander")
    assert "enabled" not in wander, wander

    driver.absent("Shell/Panel/trunk/wander")
    driver.click("Shell/Panel/trunk/add_operator")
    driver.click("Shell/Panel/trunk/add_operator/add_wander")
    driver.present("Shell/Panel/trunk/wander/angle")
    driver.screenshot("trunk_wander")
    driver.click("Shell/Panel/trunk/wander/off")
    driver.absent("Shell/Panel/trunk/wander")
    trunk = driver.get("Plant", GROWN_FROM, at="trunk")
    assert "wander" not in trunk, trunk


def files(driver):
    driver.expect(NAME, TEXT, "Untitled")
    driver.click(MENU)
    for item in ("new", "open", "save", "save_as"):
        driver.present(f"{MENU}/{item}")
    driver.screenshot("file_menu")
    driver.key("Escape")

    # A space in the name, so the buffer's and texture's URIs are escaped.
    path = os.path.join(tempfile.mkdtemp(), "live oak.gltf")
    driver.message(REQUEST, {"SaveAs": path})
    driver.expect(NAME, TEXT, "live oak")
    gltf = load(path)
    assert gltf["scenes"][0]["name"] == "live oak", gltf["scenes"]
    assert gltf["buffers"][0]["uri"] == "live%20oak.bin", gltf["buffers"]
    assert gltf["images"][0]["uri"] == "live%20oak.png", gltf["images"]
    bin_size = os.path.getsize(path.replace(".gltf", ".bin"))
    assert gltf["buffers"][0]["byteLength"] == bin_size, (gltf["buffers"], bin_size)
    assert os.path.exists(path.replace(".gltf", ".png"))
    assert "outline" in gltf["materials"][0]["extras"], gltf["materials"]

    driver.set_resource(SPECIES, 7.0, at="trunk.height")
    driver.expect(NAME, TEXT, "live oak •")
    driver.press("SuperLeft")
    driver.key("KeyS")
    driver.release("SuperLeft")
    driver.expect(NAME, TEXT, "live oak")
    height = load(path)["scenes"][0]["extras"]["arbor_day"]["trunk"]["height"]
    assert height == Between(6.999, 7.001), height

    new(driver)
    driver.expect(NAME, TEXT, "Untitled")
    driver.expect("Plant", GROWN_FROM, Between(4.999, 5.001), at="trunk.height")

    driver.message(REQUEST, {"Open": path})
    driver.expect(NAME, TEXT, "live oak")
    driver.expect("Plant", GROWN_FROM, Between(6.999, 7.001), at="trunk.height")

    # A file from before a schema change: what the species cannot take is pruned.
    stale = load(path)
    species = stale["scenes"][0]["extras"]["arbor_day"]
    species["roots"] = True
    species["trunk"]["radius"] = "thick"
    with open(path, "w") as file:
        json.dump(stale, file)
    new(driver)
    driver.message(REQUEST, {"Open": path})
    driver.expect(NAME, TEXT, "live oak •")
    driver.expect("Plant", GROWN_FROM, Between(6.999, 7.001), at="trunk.height")
    driver.expect("Plant", GROWN_FROM, Between(0.199, 0.201), at="trunk.radius")


def load(path):
    with open(path) as file:
        return json.load(file)
