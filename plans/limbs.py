"""One level of limbs on the trunk, reaching toward the envelope: more limbs per node and fewer
nodes rebuild the plant, and limbs whose start lies below the crown are left out."""

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"


def set_number(driver, path, text):
    """Replaces a number input's text. A click places the cursor rather than selecting, so clear
    what is there first."""
    driver.click(f"Shell/Panel/limbs[0]/{path}/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type(text)
    driver.key("Enter")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/limbs[0]/nodes/input")  # the immediate-mode panel has built its rows
    driver.expect("Plant", STATS, {"limbs": 8})
    driver.screenshot("spiral")
    set_number(driver, "per_node", "3")
    driver.expect("Plant", STATS, {"limbs": 24})
    driver.screenshot("whorled")
    set_number(driver, "nodes", "2")
    driver.expect("Plant", STATS, {"limbs": 6})
    # Nodes from the trunk's foot: the lower of the two sits under the crown base.
    set_number(driver, "start", "0")
    driver.expect("Plant", STATS, {"limbs": 3})
    driver.screenshot("below_the_crown")
    set_number(driver, "nodes", "0")
    driver.expect("Plant", STATS, {"limbs": 0})
    driver.exit()
