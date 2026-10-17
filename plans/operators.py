"""Operators: the ✕ on a level's Wander turns it off, hiding its rows and leaving it out of the
species as the file writes it; the level header's add menu lists it while off, and turns it back
on with the value it had. The eye beside the ✕ disables it, its rows kept and the file marking it,
and enables it again. The trunk's Wander has the same ✕ and menu."""

from driver import Between

BIN = "arbor_day"
GROWN_FROM = "arbor_day::plant::GrownFrom"
SPECIES = "arbor_day::species::Species"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.present("Shell/Panel/levels[0]/wander/angle")
    driver.set_resource(SPECIES, 0.5, at="limbs.levels[0].wander.settings.angle")
    driver.expect("Plant", GROWN_FROM, Between(0.499, 0.501), at="limbs[0].wander.angle")

    driver.click("Shell/Panel/levels[0]/wander/off")
    driver.absent("Shell/Panel/levels[0]/wander")
    level = driver.get("Plant", GROWN_FROM, at="limbs[0]")
    assert "wander" not in level, level
    driver.screenshot("wander_off")

    driver.click("Shell/Panel/levels[0]/add_operator")
    driver.screenshot("add_menu")
    driver.click("Shell/Panel/levels[0]/add_operator/add_wander")
    driver.present("Shell/Panel/levels[0]/wander/angle")
    driver.expect("Plant", GROWN_FROM, Between(0.499, 0.501), at="limbs[0].wander.angle")

    driver.click("Shell/Panel/levels[0]/wander/enabled")
    driver.expect("Plant", GROWN_FROM, False, at="limbs[0].wander.enabled")
    driver.present("Shell/Panel/levels[0]/wander/angle")
    driver.screenshot("wander_disabled")
    driver.click("Shell/Panel/levels[0]/wander/enabled")
    driver.expect("Plant", GROWN_FROM, Between(0.499, 0.501), at="limbs[0].wander.angle")
    wander = driver.get("Plant", GROWN_FROM, at="limbs[0].wander")
    assert "enabled" not in wander, wander

    driver.click("Shell/Panel/trunk/wander/off")
    driver.absent("Shell/Panel/trunk/wander")
    trunk = driver.get("Plant", GROWN_FROM, at="trunk")
    assert "wander" not in trunk, trunk
    driver.click("Shell/Panel/trunk/add_operator")
    driver.click("Shell/Panel/trunk/add_operator/add_wander")
    driver.present("Shell/Panel/trunk/wander/angle")
    driver.screenshot("trunk_wander")
    driver.exit()
