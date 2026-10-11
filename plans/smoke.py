"""The shell starts: the panel's slider shows the test object's scale, and the view draws it."""

BIN = "arbor_day"
SHAPE = "arbor_day::TestShape"
SLIDER_VALUE = "bevy_ui_widgets::slider::SliderValue"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/Scale")  # the immediate-mode panel has built its slider
    driver.expect("TestObject", SHAPE, {"scale": 1.0})
    driver.expect("Shell/Panel/Scale", SLIDER_VALUE, 1.0)
    driver.screenshot("shell")
    driver.exit()
