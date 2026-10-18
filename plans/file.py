"""File commands: the header row names the document "Untitled" and its file menu lists New, Open,
Save and Save As. Given a path (Save As waits on the file browser, X2), the row names the document
after its file, an edit marks it with a dot, and New from the menu returns to the default species,
untitled. Saving waits on the glTF writer (C41)."""

from driver import Between

BIN = "arbor_day"
DOCUMENT = "arbor_day::document::Document"
SPECIES = "arbor_day::species::Species"
GROWN_FROM = "arbor_day::plant::GrownFrom"
TEXT = "bevy_ui::widget::text::Text"
NAME = "Shell/Panel/document/name"


def run(driver):
    driver.present("Shell", ready=True)  # wait for the UI scene to finish spawning
    driver.expect(NAME, TEXT, "Untitled")

    driver.click("Shell/Panel/document/file_menu")
    for item in ("new", "open", "save", "save_as"):
        driver.present(f"Shell/Panel/document/file_menu/{item}")
    driver.screenshot("file_menu")
    driver.key("Escape")

    driver.set_resource(DOCUMENT, "/tmp/oak.gltf", at="path")
    driver.expect(NAME, TEXT, "oak")
    driver.set_resource(SPECIES, 7.0, at="trunk.height")
    driver.expect(NAME, TEXT, "oak •")

    driver.click("Shell/Panel/document/file_menu")
    driver.click("Shell/Panel/document/file_menu/new")
    driver.expect(NAME, TEXT, "Untitled")
    driver.expect("Plant", GROWN_FROM, Between(4.999, 5.001), at="trunk.height")
    driver.exit()
