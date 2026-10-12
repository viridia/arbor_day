"""The shell starts from default settings: the panel shows the trunk's parameters, the panel has
its default width, the turntable camera frames the plant, the zoom and frame keys work, the
property editor edits booleans and numbers within their limits, and an edit rebuilds the plant."""

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"
PLACEHOLDER = "arbor_day::Placeholder"
TURNTABLE = "arbor_day::turntable::TurntableCamera"
PANE = "bevy_ui_widgets::split_pane::Pane"
VISIBILITY = "bevy_camera::visibility::Visibility"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/sides/input")  # the immediate-mode panel has built its rows
    driver.present("Shell/Panel/Scrollbar")
    # The default trunk: 6 sides, bent 15°, so one segment of 6 quads and a 6-triangle cap.
    driver.expect("Plant", STATS, {"triangles": 18})
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
    driver.click("Shell/Panel/show_ground")
    driver.expect("Ground", VISIBILITY, "Hidden")
    driver.screenshot("no_ground")
    driver.click("Shell/Panel/show_ground")
    driver.expect("Ground", VISIBILITY, "Inherited")
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
    # Editing the species rebuilds the plant: 4 sides is 8 quad triangles and a 4-triangle cap.
    driver.click("Shell/Panel/sides/input")
    driver.key("End")
    driver.key("Backspace")
    driver.type("4")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"triangles": 12})
    driver.screenshot("four_sides")
    # The camera does not follow the subject: a taller crown grows out of the frame (compare the
    # screenshots), until Frame fits it again.
    driver.click("Shell/Panel/envelope/height/input")
    driver.key("End")
    driver.key("Backspace")
    driver.type("12")
    driver.key("Enter")
    driver.frames(2)
    driver.screenshot("taller")
    driver.click("Shell/Panel/Frame")
    driver.expect("SceneCamera", TURNTABLE, framed)
    driver.screenshot("taller_framed")
    driver.exit()
