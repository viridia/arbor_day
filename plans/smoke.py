"""The shell starts from default settings: the panel shows the test object's scale, the panel has
its default width, the turntable camera frames the object, the zoom and frame keys work, and the
property editor edits booleans and numbers within their limits."""

BIN = "arbor_day"
SHAPE = "arbor_day::TestShape"
PLACEHOLDER = "arbor_day::Placeholder"
TURNTABLE = "arbor_day::turntable::TurntableCamera"
PANE = "bevy_ui_widgets::split_pane::Pane"
VISIBILITY = "bevy_camera::visibility::Visibility"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/scale/input")  # the immediate-mode panel has built its rows
    driver.present("Shell/Panel/Scrollbar")
    driver.expect("TestObject", SHAPE, {"scale": 1.0})
    # The default width, whatever the developer last dragged it to: a run gets fresh settings.
    driver.expect("Shell/Panel", PANE, {"size": 320.0})
    # The view is never saved: every start frames the subject, from the front.
    driver.expect("SceneCamera", TURNTABLE, {"keep_framed": True, "azimuth": 0.0})
    driver.click("Shell/Panel/Frame")
    driver.expect("SceneCamera", TURNTABLE, {"keep_framed": True})
    driver.screenshot("shell")
    # Zooming by key lets go of the framing; F takes it back.
    driver.key("Minus")
    driver.key("Minus")
    driver.expect("SceneCamera", TURNTABLE, {"keep_framed": False})
    driver.screenshot("zoomed_out")
    driver.key("KeyF")
    driver.expect("SceneCamera", TURNTABLE, {"keep_framed": True})
    # The property editor: a checkbox per boolean field, a number input per number, a label for
    # the rest.
    driver.present("Shell/Panel/nested")
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
    driver.exit()
