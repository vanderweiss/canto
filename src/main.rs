use bevy::{
    app::AppExit,
    asset::{AssetIo, AssetIoError, ChangeWatcher, Metadata},
    prelude::*,
};

use ignore::WalkBuilder;

use std::{
    env, io,
    path::{Path, PathBuf},
};

struct Setup;
struct Keybinds;

#[derive(Component)]
struct Root;

#[derive(Component)]
struct Picture;

#[derive(Component)]
struct Frame;

#[derive(Component)]
struct Content;

fn config(mut commands: Commands) {
    let root = NodeBundle {
        style: Style {
            display: Display::Flex,
            flex_wrap: FlexWrap::Wrap,
            height: Val::Percent(100.),
            width: Val::Percent(100.),
            justify_content: JustifyContent::SpaceEvenly,
            ..default()
        },
        background_color: BackgroundColor(Color::WHITE),
        ..default()
    };

    let status = NodeBundle {
        style: Style {
            display: Display::Flex,
            ..default()
        },
        background_color: BackgroundColor(Color::BLACK),
        ..default()
    };

    commands.spawn((root, Root));
    commands.spawn(Camera2dBundle::default());
}

impl Plugin for Setup {
    fn build(&self, app: &mut App) {
        let asset = AssetPlugin {
            asset_folder: ".".to_string(),
            ..default()
        };

        let window = WindowPlugin {
            primary_window: Some(Window {
                resolution: [1000.0, 550.0].into(),
                title: "Canto".to_string(),
                resizable: false,
                ..default()
            }),
            ..default()
        };

        let edit = DefaultPlugins.set(asset).set(window);

        app.insert_resource(ClearColor(Color::DARK_GRAY))
            .add_plugins(edit)
            .add_systems(Startup, config);
    }
}

fn render(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input: Res<Input<KeyCode>>,
    mut gallery: ResMut<Gallery>,
    mut query: Query<Entity, With<Root>>,
) {
    if input.just_pressed(KeyCode::Space) {
        match gallery.layout {
            Layout::Grid(dim) => {
                let path = gallery.fetch_next();
                env::set_var("BEVY_ASSET_ROOT", path.parent().unwrap());

                let img: Handle<Image> =
                    asset_server.load(path.file_name().unwrap().to_str().unwrap());

                commands.entity(query.single_mut()).with_children(|pb| {
                    pb.spawn((
                        NodeBundle {
                            style: Style {
                                display: Display::Flex,
                                width: Val::Percent(17.5),
                                max_width: Val::Percent(17.5),
                                height: Val::Percent(17.5),
                                max_height: Val::Percent(17.5),
                                justify_content: JustifyContent::Center,
                                margin: UiRect::all(Val::Percent(1.)),
                                ..default()
                            },
                            ..default()
                        },
                        Picture,
                    ))
                    .with_children(|fb| {
                        fb.spawn((
                            NodeBundle {
                                style: Style {
                                    display: Display::Flex,
                                    width: Val::Percent(100.),
                                    max_width: Val::Percent(100.),
                                    height: Val::Percent(100.),
                                    max_height: Val::Percent(100.),
                                    border: UiRect::all(Val::Px(1.)),
                                    ..default()
                                },
                                border_color: BorderColor(Color::BLACK),
                                ..default()
                            },
                            Frame,
                        ))
                        .with_children(|cb| {
                            cb.spawn((
                                ImageBundle {
                                    image: UiImage::new(img),
                                    style: Style {
                                        max_width: Val::Percent(100.),
                                        max_height: Val::Percent(100.),
                                        ..default()
                                    },
                                    ..default()
                                },
                                Content,
                            ));
                        });
                    });
                });
            }
            Layout::Slide => {}
            Layout::Opt => return,
        }
    }
}

fn quit(input: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    let shift = input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if shift && input.just_pressed(KeyCode::Q) {
        exit.send(AppExit)
    }
}

impl Plugin for Keybinds {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, quit);
    }
}

enum Layout {
    Grid(u16),
    Slide,
    Opt,
}

#[derive(Resource)]
struct Gallery {
    root: Vec<PathBuf>,
    layout: Layout,
    position: usize,
}

impl Gallery {
    #[inline]
    fn in_bound(&self) -> bool {
        (self.root.len() - 1) != self.position && self.position != 0
    }

    fn jump_next() {}

    fn jump_previous() {}

    fn fetch_next(&mut self) -> &PathBuf {
        if self.in_bound() {
            self.position += 1;
        } else {
            self.position = 1;
        }
        &self.root[self.position]
    }

    fn pfetch_next(&mut self, step: u16) {}

    fn fetch_previous(&mut self) -> &PathBuf {
        if self.in_bound() {
            self.position -= 1;
        } else {
            self.position = self.root.len() - 1;
        }
        &self.root[self.position]
    }

    fn pfetch_previous(&mut self, step: u16) {}
}

fn main() -> io::Result<()> {
    let mut args: Vec<String> = env::args().collect();
    let cwd = env::current_dir()?;

    // temp
    args.remove(0);
    args.pop();
    args.pop();

    let mut root: Vec<PathBuf> = Vec::new();

    let mut builder = {
        let mut opt: Option<WalkBuilder> = None;
        if args.is_empty() {
            opt = Some(WalkBuilder::new(cwd));
        } else {
            for arg in args {
                let pre = Path::new(arg.as_str());
                let mut post = PathBuf::new();

                if pre.is_absolute() {
                    post = post.join(pre);
                } else {
                    post = post.join(cwd.join(pre));
                }

                if post.is_dir() {
                    if let Some(mut _builder) = opt.as_mut() {
                        _builder.add(post);
                    } else {
                        opt = Some(WalkBuilder::new(post));
                    }
                } else {
                    root.push(post);
                }
            }
        }
        opt.unwrap()
    };

    for result in builder
        .git_ignore(false)
        .hidden(false)
        .max_depth(Some(3))
        .build()
    {
        match result {
            Ok(entry) => {
                root.push(entry.into_path());
            }
            Err(_) => continue,
        }
    }

    let gallery = Gallery {
        root,
        layout: Layout::Grid(4),
        position: 0,
    };

    App::new()
        .insert_resource(gallery)
        .add_plugins((Setup, Keybinds))
        .add_systems(Update, render)
        .run();

    Ok(())
}
