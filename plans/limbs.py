"""One level of limbs on the trunk, reaching toward the envelope: more limbs per node and fewer
nodes rebuild the plant. The trunk inside the crown is about 1.25 crown radii, so the node count
is about 1.25 times the density."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"


def set_number(driver, path, text, input="input"):
    """Replaces a number input's text; a fuzzy row's center input is `center`. A click places the
    cursor rather than selecting, so clear what is there first."""
    driver.click(f"Shell/Panel/limbs[0]/{path}/{input}")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type(text)
    driver.key("Enter")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/limbs[0]/density/input")  # the immediate-mode panel has built its rows
    driver.expect("Plant", STATS, {"limbs": Between(9, 10)})
    driver.screenshot("spiral")
    set_number(driver, "per_node", "3")
    driver.expect("Plant", STATS, {"limbs": Between(27, 30)})
    driver.screenshot("whorled")
    set_number(driver, "density", "2")
    driver.expect("Plant", STATS, {"limbs": Between(6, 9)})
    # One or two limbs at each of two or three nodes.
    set_number(driver, "per_node", "1.5")
    driver.expect("Plant", STATS, {"limbs": Between(2, 6)})
    driver.screenshot("mixed")
    set_number(driver, "density", "0")
    driver.expect("Plant", STATS, {"limbs": 0})
    driver.exit()
