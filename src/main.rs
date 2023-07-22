use bevy::{app::AppExit, ecs::system::EntityCommands, prelude::*};

use ignore::WalkBuilder;

use std::{
    env, io,
    path::{Path, PathBuf},
};

struct Setup;
struct Keybinds;

#[derive(Component)]
struct Watchdog;

#[derive(Component)]
struct Root;

#[derive(Component)]
struct Media {
    selected: bool,
}

#[derive(Component)]
struct Frame;

#[derive(Component)]
struct Content;

enum Layout {
    Grid(u16),
    Slide,
    Opt,
}

#[derive(Resource)]
struct Gallery {
    pre: Vec<PathBuf>,
    post: Vec<Option<Handle<Image>>>,
    layout: Layout,
    position: usize,
}

impl Gallery {
    #[inline]
    fn in_bound(&self) -> bool {
        (self.pre.len() - 1) != self.position && self.position != 0
    }

    #[inline]
    fn in_range(&self, step: u16) -> bool {
        !((self.pre.len() - step as usize) <= self.position)
    }

    #[inline]
    fn valid(&self) -> bool {
        self.pre.len() == self.post.len()
    }

    fn fetch_next(&mut self) -> &PathBuf {
        if self.in_bound() {
            self.position += 1;
        } else {
            self.position = 1;
        }
        &self.pre[self.position]
    }

    fn pfetch_next(&mut self, step: u16) {}

    fn fetch_previous(&mut self) -> &PathBuf {
        if self.in_bound() {
            self.position -= 1;
        } else {
            self.position = self.pre.len() - 1;
        }
        &self.pre[self.position]
    }

    fn pfetch_previous(&mut self, step: u16) {}
}

#[derive(Resource)]
struct Selection {
    media: Handle<Image>,
}

fn config(mut commands: Commands) {
    let root = NodeBundle {
        style: Style {
            display: Display::Flex,
            top: Val::Px(0.),
            bottom: Val::Px(0.),
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

    commands.spawn((Camera2dBundle::default(), Watchdog));
    commands.spawn((root, Root));
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

        app.insert_resource(ClearColor(Color::WHITE))
            .add_plugins(edit)
            .add_systems(Startup, config);
    }
}

fn insert_media(mut commands: EntityCommands, handle: Handle<Image>) {
    commands.with_children(|pb| {
        pb.spawn((
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    width: Val::Percent(15.),
                    max_width: Val::Percent(15.),
                    height: Val::Percent(15.),
                    max_height: Val::Percent(15.),
                    justify_content: JustifyContent::Center,
                    margin: UiRect::all(Val::Percent(1.)),
                    ..default()
                },
                ..default()
            },
            Media { selected: false },
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
                    background_color: BackgroundColor(Color::WHITE),
                    //border_color: BorderColor(Color::BLACK),
                    ..default()
                },
                Frame,
            ))
            .with_children(|cb| {
                cb.spawn((
                    ImageBundle {
                        image: UiImage::new(handle),
                        style: Style {
                            max_width: Val::Percent(90.),
                            max_height: Val::Percent(90.),
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

fn render(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input: Res<Input<KeyCode>>,
    mut gallery: ResMut<Gallery>,
    mut query: Query<Entity, With<Root>>,
) {
    if input.just_pressed(KeyCode::Space) {
        for i in 1..gallery.pre.len() {
            match gallery.layout {
                Layout::Grid(_) => {
                    let path = &gallery.pre[i];
                    env::set_var("BEVY_ASSET_ROOT", path.parent().unwrap());

                    let handle: Handle<Image> =
                        asset_server.load(path.file_name().unwrap().to_str().unwrap());

                    let entity = query.single_mut();

                    insert_media(commands.entity(entity), handle);
                }
                Layout::Slide => {}
                Layout::Opt => return,
            }
        }
    }
}

fn switch(input: Res<Input<KeyCode>>, mut query: Query<&mut Style, With<Root>>) {
    let mut camera = query.single_mut();

    let shift = input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if shift && input.pressed(KeyCode::Up) {
        let _ = camera.top.try_add_assign(Val::Px(5.));
    }
    if shift && input.pressed(KeyCode::Down) {
        let _ = camera.top.try_sub_assign(Val::Px(5.));
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
        app.add_systems(Update, switch).add_systems(Update, quit);
    }
}

fn main() -> io::Result<()> {
    let mut args: Vec<String> = env::args().collect();
    let cwd = env::current_dir()?;

    // temp
    args.remove(0);

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
        pre: root,
        post: Vec::new(),
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
