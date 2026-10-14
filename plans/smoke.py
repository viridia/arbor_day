"""The shell starts from default settings: the panel shows the trunk's parameters, the panel has
its default width, the turntable camera frames the plant, the zoom and frame keys work, the HUD's
toggles hide the ground and envelope, the property editor edits booleans and numbers within their
limits, an edit rebuilds the plant, and
the seed reaches the plant, typed or picked by Random."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"
PLACEHOLDER = "arbor_day::Placeholder"
TURNTABLE = "arbor_day::turntable::TurntableCamera"
PANE = "bevy_ui_widgets::split_pane::Pane"
VISIBILITY = "bevy_camera::visibility::Visibility"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/trunk/sides/input")  # the immediate-mode panel has built its rows
    driver.present("Shell/Panel/Scrollbar")
    # The default plant: one level of nine or ten limbs, in the low hundreds of triangles.
    driver.expect(
        "Plant", STATS, {"seed": 1, "limbs": Between(9, 10), "triangles": Between(100, 1000)}
    )
    # The default width, whatever the developer last dragged it to: a run gets fresh settings.
    driver.expect("Shell/Panel", PANE, {"size": 320.0})
    # The view is never saved: every start frames the subject, from the front.
    framed = {"frame_pending": False, "keep_framed": True}
    driver.expect("SceneCamera", TURNTABLE, {**framed, "azimuth": 0.0})
    driver.click("Shell/Panel/Frame")
    driver.expect("SceneCamera", TURNTABLE, framed)
    driver.screenshot("shell")
    # Zooming by key lets go of the framing; F takes it back.
    driver.key("Minus")
    driver.key("Minus")
    driver.expect("SceneCamera", TURNTABLE, {"keep_framed": False})
    driver.screenshot("zoomed_out")
    driver.key("KeyF")
    driver.expect("SceneCamera", TURNTABLE, framed)
    # The property editor: a checkbox per boolean field, a number input per number, a subpane of
    # rows per nested struct.
    driver.present("Shell/Panel/trunk/sides/input")
    driver.present("Shell/Panel/nested/size/input")
    driver.click("Shell/Panel/nested/flag")
    driver.expect("Placeholder", PLACEHOLDER, {"nested": {"flag": True}})
    driver.screenshot("nested")
    # The HUD over the view: the triangle count, and toggles that hide the ground and envelope.
    driver.present("Shell/View/triangles")
    driver.click("Shell/View/ground")
    driver.expect("Ground", VISIBILITY, "Hidden")
    driver.click("Shell/View/envelope")
    driver.expect("Envelope", VISIBILITY, "Hidden")
    driver.screenshot("hud_hidden")
    driver.click("Shell/View/ground")
    driver.expect("Ground", VISIBILITY, "Inherited")
    driver.click("Shell/View/envelope")
    driver.expect("Envelope", VISIBILITY, "Inherited")
    # Typing past `count`'s range (0 to 10) stops at its hard limit. A click places the cursor
    # rather than selecting, so clear the default (3) first.
    driver.expect("Placeholder", PLACEHOLDER, {"count": 3})
    driver.click("Shell/Panel/count/input")
    driver.key("End")
    driver.key("Backspace")
    driver.type("42")
    driver.key("Enter")
    driver.expect("Placeholder", PLACEHOLDER, {"count": 10})
    driver.screenshot("numbers")
    # Editing the species rebuilds the plant: four or five nodes on the trunk's 1.2 crown radii
    # inside the crown.
    driver.click("Shell/Panel/limbs[0]/density/input")
    driver.key("End")
    driver.key("Backspace")
    driver.type("4")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"limbs": Between(4, 5)})
    driver.screenshot("four_nodes")
    # The camera does not follow the subject: a taller tree grows out of the frame (compare the
    # screenshots), until Frame fits it again.
    driver.click("Shell/Panel/trunk/height/input")
    driver.key("End")
    driver.key("Backspace")
    driver.type("12")
    driver.key("Enter")
    driver.frames(2)
    driver.screenshot("taller")
    driver.click("Shell/Panel/Frame")
    driver.expect("SceneCamera", TURNTABLE, framed)
    driver.screenshot("taller_framed")
    # The seed: a typed one reaches the plant, and Random picks another.
    driver.click("Shell/Panel/seed/input")
    driver.key("End")
    for _ in range(8):
        driver.key("Backspace")
    driver.type("42")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"seed": 42})
    driver.screenshot("seed_42")
    driver.click("Shell/Panel/seed/randomize")
    driver.frames(2)
    driver.screenshot("random_seed")
    driver.exit()
