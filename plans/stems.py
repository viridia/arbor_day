"""Stems: several stems splayed from up, each bearing the first level of limbs, and the leader
setting the stems' length as a share of the envelope's height."""

from driver import Between

BIN = "arbor_day"
STATS = "arbor_day::plant::PlantStats"


def set_number(driver, path, text):
    """Replaces a trunk field's text. A click places the cursor rather than selecting, so clear
    what is there first."""
    driver.click(f"Shell/Panel/trunk/{path}/input")
    driver.key("End")
    for _ in range(6):
        driver.key("Backspace")
    driver.type(text)
    driver.key("Enter")


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/trunk/stems/input")  # the immediate-mode panel has built its rows
    driver.expect("Plant", STATS, {"limbs": 8})

    # Three stems, each with limbs of its own, less those starting outside the envelope.
    set_number(driver, "stems", "3")
    set_number(driver, "splay", "25")
    driver.expect("Plant", STATS, {"limbs": Between(9, 3 * 8)})
    driver.screenshot("three_stems")

    # Shorter stems bear fewer limbs: with the crown starting 30% of the way up, the nodes of
    # half-height splayed stems lie under it or outside its narrow foot, so few if any remain.
    set_number(driver, "leader", "50")
    driver.expect("Plant", STATS, {"limbs": Between(0, 3 * 8 - 1)})
    driver.screenshot("short_leader")
    driver.exit()
