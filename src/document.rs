//! The document: the species being edited and the file it was saved to, and the file commands
//! (New, Open, Save, Save As), from the keys (`keys.rs`) and the file menu on the panel's header
//! row. The saved file is the exported glTF, the species a JSON extra on its scene
//! (D34, `gltf_file`). Open and Save As pick the file in a native dialog (`rfd`), run as a task
//! so the app keeps drawing while it is open.

use std::path::PathBuf;

use bevy::{
    feathers::controls::{FeathersMenu, FeathersMenuDivider, FeathersMenuToolButton},
    feathers::display::{icon, label},
    prelude::*,
    tasks::{IoTaskPool, Task, futures::check_ready},
    window::PrimaryWindow,
};
use bevy_immediate::{Imm, ui::text::ImmUiText};
use rfd::{AsyncFileDialog, FileHandle};

use crate::{
    gltf_file,
    properties::{CapsPanel, menu_item, right_aligned_popup},
    species::Species,
};

pub struct DocumentPlugin;

impl Plugin for DocumentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Document>()
            .add_message::<FileRequest>()
            .add_observer(on_file_command)
            .add_systems(
                Update,
                (
                    finish_dialog.run_if(resource_exists::<OpenDialog>),
                    on_file_request,
                    show_title,
                )
                    .chain(),
            );
    }
}

/// The open document's file, and the species as last saved, so unsaved changes show.
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Document {
    /// The file the species was last saved to; none until the first save.
    pub path: Option<PathBuf>,
    saved: Species,
    /// Opened with fields its file held pruned, so the file differs from the species until saved.
    pruned: bool,
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
        self.pruned || *species != self.saved
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

/// A native file dialog the user has open, and the command it is for.
#[derive(Resource)]
struct OpenDialog {
    task: Task<Option<FileHandle>>,
    command: FileCommand,
}

fn on_file_command(
    command: On<FileCommand>,
    mut document: ResMut<Document>,
    mut species: ResMut<Species>,
    dialog: Option<Res<OpenDialog>>,
    registry: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    // A dialog already open takes every command until it closes.
    if dialog.is_some() {
        return;
    }
    let gltf = || AsyncFileDialog::new().add_filter("glTF", &["gltf"]);
    match *command {
        // Unsaved changes are dropped without asking (ledger C108).
        FileCommand::New => {
            *species = Species::default();
            *document = Document::default();
        }
        FileCommand::Save if let Some(path) = &document.path => {
            match gltf_file::write(path, &document.name(), &species, &registry.read()) {
                Ok(()) => {
                    document.saved = species.clone();
                    document.pruned = false;
                }
                Err(error) => error!("Not saved to {}: {error}", path.display()),
            }
        }
        FileCommand::Open => {
            let pick = gltf().pick_file();
            commands.insert_resource(OpenDialog {
                task: IoTaskPool::get().spawn(pick),
                command: FileCommand::Open,
            });
        }
        FileCommand::Save | FileCommand::SaveAs => {
            let pick = gltf()
                .set_file_name(format!("{}.gltf", document.name()))
                .save_file();
            commands.insert_resource(OpenDialog {
                task: IoTaskPool::get().spawn(pick),
                command: FileCommand::SaveAs,
            });
        }
    }
}

/// A file to open or save as, by path: picked in a dialog, or sent by a plan through the driver's
/// `message`, which has no dialog to work.
#[derive(Message, Reflect, Clone, Debug)]
#[reflect(Message)]
pub enum FileRequest {
    Open(PathBuf),
    SaveAs(PathBuf),
}

/// Requests the file picked in the open dialog once it closes. Nothing happens if the user
/// cancels.
fn finish_dialog(
    mut dialog: ResMut<OpenDialog>,
    mut requests: MessageWriter<FileRequest>,
    mut commands: Commands,
) {
    let Some(picked) = check_ready(&mut dialog.task) else {
        return;
    };
    commands.remove_resource::<OpenDialog>();
    let Some(file) = picked else {
        return;
    };
    let path = file.path().to_path_buf();
    requests.write(match dialog.command {
        FileCommand::SaveAs => FileRequest::SaveAs(path),
        _ => FileRequest::Open(path),
    });
}

/// Save As takes its path as the document's and saves; Open reads the species from its file.
fn on_file_request(
    mut requests: MessageReader<FileRequest>,
    mut document: ResMut<Document>,
    mut species: ResMut<Species>,
    registry: Res<AppTypeRegistry>,
    mut commands: Commands,
) {
    for request in requests.read() {
        match request {
            FileRequest::SaveAs(path) => {
                document.path = Some(path.clone());
                commands.trigger(FileCommand::Save);
            }
            // Unsaved changes are dropped without asking (ledger C108).
            FileRequest::Open(path) => match gltf_file::read(path, &registry.read()) {
                Ok(opened) => {
                    if !opened.pruned.is_empty() {
                        warn!(
                            "{} opened with defaults in place of:\n  {}",
                            path.display(),
                            opened.pruned.join("\n  ")
                        );
                    }
                    *species = opened.species.clone();
                    *document = Document {
                        path: Some(path.clone()),
                        saved: opened.species,
                        pruned: !opened.pruned.is_empty(),
                    };
                }
                Err(error) => error!("{} not opened: {error}", path.display()),
            },
        }
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
