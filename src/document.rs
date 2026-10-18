//! The document: the species being edited and the file it was saved to, and the file commands
//! (New, Open, Save, Save As), from the keys (`keys.rs`) and the file menu on the panel's header
//! row. The saved file is the exported glTF, the species a JSON extra on its scene
//! (requirements, Parameters). Open and Save As wait on the file browser (X2), and Save on the
//! glTF writer (C41); until then they only log.

use std::path::PathBuf;

use bevy::{
    feathers::controls::{FeathersMenu, FeathersMenuDivider, FeathersMenuToolButton},
    feathers::display::{icon, label},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_immediate::{Imm, ui::text::ImmUiText};

use crate::{
    properties::{CapsPanel, menu_item, right_aligned_popup},
    species::Species,
};

pub struct DocumentPlugin;

impl Plugin for DocumentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Document>()
            .add_observer(on_file_command)
            .add_systems(Update, show_title);
    }
}

/// The open document's file, and the species as last saved, so unsaved changes show.
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Document {
    /// The file the species was last saved to; none until the first save.
    pub path: Option<PathBuf>,
    saved: Species,
}

impl Document {
    /// The document's name: its file's name without the extension, or "Untitled" before the
    /// first save.
    pub fn name(&self) -> String {
        (self.path.as_ref())
            .and_then(|path| path.file_stem())
            .map_or("Untitled".into(), |stem| {
                stem.to_string_lossy().into_owned()
            })
    }

    /// Whether `species` differs from the species as last saved.
    pub fn edited(&self, species: &Species) -> bool {
        *species != self.saved
    }

    /// The name, and a dot while `species` has unsaved changes.
    fn title(&self, species: &Species) -> String {
        let dot = if self.edited(species) { " •" } else { "" };
        format!("{}{dot}", self.name())
    }
}

/// A command on the open document.
#[derive(Event, Clone, Copy, Debug)]
pub enum FileCommand {
    New,
    Open,
    Save,
    SaveAs,
}

fn on_file_command(
    command: On<FileCommand>,
    mut document: ResMut<Document>,
    mut species: ResMut<Species>,
) {
    match *command {
        // Unsaved changes are dropped without asking (ledger C108).
        FileCommand::New => {
            *species = Species::default();
            *document = Document::default();
        }
        FileCommand::Open => warn!("Open needs the file browser (X2)"),
        FileCommand::Save if document.path.is_some() => {
            warn!("Save needs the glTF writer (C41)");
        }
        FileCommand::Save | FileCommand::SaveAs => warn!("Save As needs the file browser (X2)"),
    }
}

/// The window's title: the document's name and its unsaved dot.
fn show_title(
    document: Res<Document>,
    species: Res<Species>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if !document.is_changed() && !species.is_changed() {
        return;
    }
    window.title = format!("{} — arbor_day", document.title(&species));
}

/// A shortcut as the platform writes it: ⌘ and ⇧ on macOS, Ctrl+ and Shift+ elsewhere.
const fn keys(mac: &'static str, other: &'static str) -> &'static str {
    if cfg!(target_os = "macos") {
        mac
    } else {
        other
    }
}

/// The panel's header row, named `document`: the document's name and its unsaved dot, named
/// `name`, and at the right the file menu, named `file_menu`, its items named `new`, `open`,
/// `save` and `save_as`. Returns the command picked.
pub fn header(
    ui: &mut Imm<CapsPanel>,
    document: &Document,
    species: &Species,
) -> Option<FileCommand> {
    let mut picked = None;
    let title = document.title(species);
    ui.ch_id("document")
        .on_spawn_insert(|| {
            (
                Name::new("document"),
                Node {
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
            )
        })
        .add(|ui| {
            ui.ch()
                .on_spawn_apply_scene(|| bsn! { @label("") #name })
                .on_hash_change_text_fn(&title, || title.clone());
            ui.ch()
                .on_spawn_apply_scene(|| bsn! { @FeathersMenu #file_menu })
                .add(|ui| {
                    ui.ch().on_spawn_apply_scene(|| {
                        bsn! {
                            @FeathersMenuToolButton {
                                @caption: bsn! { @icon("icons/menu.png") Node { height: px(18) } },
                                @arrow: false,
                            }
                        }
                    });
                    ui.ch().on_spawn_apply_scene(right_aligned_popup).add(|ui| {
                        let items = [
                            ("new", "New", keys("⌘N", "Ctrl+N"), FileCommand::New),
                            ("open", "Open…", keys("⌘O", "Ctrl+O"), FileCommand::Open),
                            ("save", "Save", keys("⌘S", "Ctrl+S"), FileCommand::Save),
                            (
                                "save_as",
                                "Save As…",
                                keys("⌘⇧S", "Ctrl+Shift+S"),
                                FileCommand::SaveAs,
                            ),
                        ];
                        for (name, caption, keys, command) in items {
                            if name == "save" {
                                ui.ch_id("divider")
                                    .on_spawn_apply_scene(|| bsn! { @FeathersMenuDivider });
                            }
                            if menu_item(ui.ch_id(name), caption.into(), Some(keys), name.into()) {
                                picked = Some(command);
                            }
                        }
                    });
                });
        });
    picked
}
