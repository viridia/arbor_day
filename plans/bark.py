"""The bark color: a color field's row is a swatch that opens a picker, and a color typed into the
picker tints the trunk."""

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"
INPUT = "Shell/Panel/bark/color/input"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present(INPUT)  # the immediate-mode panel has built its rows
    driver.expect("Plant", STATS, {"bark_color": "#6B5440"})
    driver.click(INPUT)
    driver.present(f"{INPUT}/wheel")
    driver.screenshot("picker")
    # The hex field shows in RGB mode. A click places the cursor, so clear the field first.
    driver.click(f"{INPUT}/mode_rgb")
    driver.click(f"{INPUT}/hex_input")
    driver.key("End")
    for _ in range(9):
        driver.key("Backspace")
    driver.type("336699")
    driver.key("Enter")
    driver.expect("Plant", STATS, {"bark_color": "#336699"})
    driver.screenshot("picked")
    driver.key("Escape")
    driver.absent(f"{INPUT}/wheel")
    driver.screenshot("closed")
    driver.exit()
