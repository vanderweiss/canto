use bevy::{app::AppExit, ecs::system::EntityCommands, prelude::*};

use ignore::WalkBuilder;

use std::{
    env, io,
    path::{Path, PathBuf},
};

const X: f32 = 1000.;
const Y: f32 = 550.;

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
    Display,
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
            margin: UiRect::all(Val::Percent(2.5)),
            row_gap: Val::Percent(5.),
            column_gap: Val::Percent(2.5),
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
                resolution: [X, Y].into(),
                title: "Canto".to_string(),
                resizable: true,
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

fn insert_media(mut commands: EntityCommands, handle: Handle<Image>, image: &Image) {
    let size = image.size();
    let (mut x, mut y) = (size.x, size.y);

    while X * 0.4 < x || Y * 0.4 < y {
        x *= 0.75;
        y *= 0.75;
    }

    commands.with_children(|pb| {
        pb.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(x),
                    height: Val::Px(y),
                    justify_content: JustifyContent::Center,
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
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
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
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
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
    assets: ResMut<Assets<Image>>,
    server: Res<AssetServer>,
    input: Res<Input<KeyCode>>,
    gallery: ResMut<Gallery>,
    mut query: Query<Entity, With<Root>>,
) {
    if input.just_pressed(KeyCode::Space) {
        for i in 0..gallery.pre.len() {
            match gallery.layout {
                Layout::Display => {
                    let path = &gallery.pre[i];
                    env::set_var("BEVY_ASSET_ROOT", path.parent().unwrap());

                    let handle: Handle<Image> =
                        server.load(path.file_name().unwrap().to_str().unwrap());

                    let fetch = assets.get(&handle);

                    if let Some(ref image) = fetch {
                        insert_media(commands.entity(query.single_mut()), handle, image);
                    } else {
                        continue;
                    }
                }
                Layout::Slide => {}
                Layout::Opt => return,
            }
        }
    }
}

fn switch(input: Res<Input<KeyCode>>, mut query: Query<&mut Style, With<Root>>) {
    let mut root = query.single_mut();

    let focus = root.top.evaluate(1.).unwrap();

    let shift = input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    if shift && input.pressed(KeyCode::Up) && focus <= 0. {
        let _ = root.top.try_add_assign(Val::Px(5.));
    }
    if shift && input.pressed(KeyCode::Down) {
        let _ = root.top.try_sub_assign(Val::Px(5.));
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
        .git_exclude(false)
        .git_global(false)
        .git_ignore(false)
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
        layout: Layout::Display,
        position: 0,
    };

    App::new()
        .insert_resource(gallery)
        .add_plugins((Setup, Keybinds))
        .add_systems(Update, render)
        .run();

    Ok(())
}
