"""The shell starts from default settings: the panel's slider shows the test object's scale, the
panel has its default width, and the turntable camera frames the object."""

BIN = "arbor_day"
SHAPE = "arbor_day::TestShape"
TURNTABLE = "arbor_day::turntable::TurntableCamera"
SLIDER_VALUE = "bevy_ui_widgets::slider::SliderValue"
PANE = "bevy_ui_widgets::split_pane::Pane"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/Scale")  # the immediate-mode panel has built its slider
    driver.expect("TestObject", SHAPE, {"scale": 1.0})
    driver.expect("Shell/Panel/Scale", SLIDER_VALUE, 1.0)
    # The default width, whatever the developer last dragged it to: a run gets fresh settings.
    driver.expect("Shell/Panel", PANE, {"size": 320.0})
    # The view is never saved: every start frames the subject, from the front.
    driver.expect("SceneCamera", TURNTABLE, {"keep_framed": True, "azimuth": 0.0})
    driver.click("Shell/Panel/Frame")
    driver.expect("SceneCamera", TURNTABLE, {"keep_framed": True})
    driver.screenshot("shell")
    driver.exit()
