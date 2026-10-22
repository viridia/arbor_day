"""The shell starts from default settings: the panel shows the trunk's parameters, the panel has
its default width, the turntable camera frames the plant, the zoom and frame keys work, the HUD's
toggles hide the ground, envelope and scale and show the wireframe, the property editor edits numbers within their limits, an
edit rebuilds the plant, and the seed reaches the plant, typed or picked by Random."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"
GROWN_FROM = "arbor_day::plant::GrownFrom"
TURNTABLE = "arbor_day::turntable::TurntableCamera"
PANE = "bevy_ui_widgets::split_pane::Pane"
VISIBILITY = "bevy_camera::visibility::Visibility"
WIREFRAME = "bevy_pbr::wireframe::Wireframe"
VIEW_SETTINGS = "arbor_day::settings::ViewSettings"


def set_number(driver, path, text):
    """Replaces a number input's text. A click places the cursor rather than selecting, so clear
    what is there first."""
    driver.click(f"Shell/Panel/{path}/input")
    driver.key("End")
    for _ in range(8):
        driver.key("Backspace")
    driver.type(text)
    driver.key("Enter")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/trunk/sides/input")  # the immediate-mode panel has built its rows
    driver.present("Shell/Panel/Scrollbar")
    # The default plant: one level of nine or ten limbs, in the low hundreds of triangles.
    driver.expect("Plant", STATS, {"limbs": Between(9, 10), "triangles": Between(100, 1000)})
    driver.expect("Plant", GROWN_FROM, 1, at="seed")
    # The default width, whatever the developer last dragged it to: a run gets fresh settings.
    driver.expect("Shell/Panel", PANE, {"size": 320.0})
    # The view is never saved: every start frames the subject, from the front.
    framed = {"frame_pending": False, "keep_framed": True}
    driver.expect("SceneCamera", TURNTABLE, {**framed, "azimuth": 0.0})
    driver.click("Shell/View/frame")
    driver.expect("SceneCamera", TURNTABLE, framed)
    driver.screenshot("shell")
    # Zooming by key lets go of the framing; F takes it back.
    driver.key("Minus")
    driver.key("Minus")
    driver.expect("SceneCamera", TURNTABLE, {"keep_framed": False})
    driver.screenshot("zoomed_out")
    driver.key("KeyF")
    driver.expect("SceneCamera", TURNTABLE, framed)
    # The HUD's camera buttons move the camera while held, by time, so only the direction is
    # checked: each reverse button is held twice as long, to cross back past where it started.
    # Orbit wraps at a full turn, and tilt keeps between 5° and 85°, from the game's 45° (0.785).
    driver.click("Shell/View/orbit_left", hold=30)
    driver.expect("SceneCamera", TURNTABLE, {"azimuth": Between(0.05, 3.0)})
    driver.click("Shell/View/orbit_right", hold=60)
    driver.expect("SceneCamera", TURNTABLE, {"azimuth": Between(3.3, 6.25)})
    driver.click("Shell/View/tilt_up", hold=30)
    driver.expect("SceneCamera", TURNTABLE, {"elevation": Between(0.8, 1.49)})
    driver.click("Shell/View/tilt_down", hold=60)
    driver.expect("SceneCamera", TURNTABLE, {"elevation": Between(0.087, 0.78)})
    # Zooming lets go of the framing, as the keys do; Frame takes it back.
    driver.click("Shell/View/zoom_in", hold=30)
    driver.expect("SceneCamera", TURNTABLE, {"keep_framed": False})
    driver.click("Shell/View/zoom_out", hold=30)
    driver.click("Shell/View/frame")
    driver.expect("SceneCamera", TURNTABLE, framed)
    # The HUD over the view: the triangle count, and toggles that hide the ground, envelope and
    # scale.
    driver.present("Shell/View/triangles")
    driver.click("Shell/View/ground")
    driver.expect("Ground", VISIBILITY, "Hidden")
    driver.click("Shell/View/envelope")
    driver.expect("Envelope", VISIBILITY, "Hidden")
    driver.click("Shell/View/scale")
    driver.expect("ScaleFigure", VISIBILITY, "Hidden")
    driver.screenshot("hud_hidden")
    driver.click("Shell/View/ground")
    driver.expect("Ground", VISIBILITY, "Inherited")
    driver.click("Shell/View/envelope")
    driver.expect("Envelope", VISIBILITY, "Inherited")
    driver.click("Shell/View/scale")
    driver.expect("ScaleFigure", VISIBILITY, "Inherited")
    # The wireframe toggle draws lines over the bark and the leaves, and its second click turns
    # the setting off again.
    driver.click("Shell/View/wireframe")
    driver.expect("Plant", WIREFRAME, {})
    driver.expect("Leaves", WIREFRAME, {})
    driver.screenshot("wireframe")
    driver.click("Shell/View/wireframe")
    driver.frames(2)
    view = driver.get_resource(VIEW_SETTINGS)
    assert not view["show_wireframe"], view
    # Typing past the trunk's sides' range (3 to 8) stops at its hard limit.
    set_number(driver, "trunk/sides", "42")
    driver.expect("Plant", GROWN_FROM, 8, at="trunk.sides")
    driver.screenshot("numbers")
    set_number(driver, "trunk/sides", "6")
    driver.expect("Plant", GROWN_FROM, 6, at="trunk.sides")
    # Editing the species rebuilds the plant: four or five nodes on the trunk's 1.2 crown radii
    # inside the crown.
    set_number(driver, "levels[0]/density", "4")
    driver.expect("Plant", STATS, {"limbs": Between(4, 5)})
    driver.screenshot("four_nodes")
    # The camera does not follow the subject: a taller tree grows out of the frame (compare the
    # screenshots), until Frame fits it again.
    set_number(driver, "trunk/height", "12")
    driver.expect("Plant", GROWN_FROM, 12.0, at="trunk.height")
    driver.screenshot("taller")
    driver.click("Shell/View/frame")
    driver.expect("SceneCamera", TURNTABLE, framed)
    driver.screenshot("taller_framed")
    # The seed: a typed one reaches the plant, and Random picks another.
    set_number(driver, "seed", "42")
    driver.expect("Plant", GROWN_FROM, 42, at="seed")
    driver.screenshot("seed_42")
    driver.click("Shell/Panel/seed/randomize")
    driver.frames(2)
    driver.screenshot("random_seed")
    driver.exit()
