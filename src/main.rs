use bevy::{
    app::{App, FixedUpdate, Startup, Update},
    color::{Color, Hue},
    input::ButtonInput,
    math::{Quat, Vec2, Vec3},
    prelude::{
        BuildChildren, Camera2dBundle, ClearColor, Commands, Component, Deref, DerefMut, Entity,
        IntoSystemConfigs, MouseButton, NodeBundle, Query, Res, ResMut, Resource, TextBundle,
        Transform, With,
    },
    sprite::{Sprite, SpriteBundle},
    text::{Text, TextSection, TextStyle},
    time::{Time, Timer, TimerMode},
    ui::{AlignItems, JustifyContent, Style, Val},
    utils::default,
    window::{PrimaryWindow, Window},
    DefaultPlugins,
};
use rand::Rng;

mod plugin;

#[derive(Component)]
struct LabelUi;

#[derive(Debug, Clone, Eq, PartialEq, Resource, Deref, DerefMut)]
struct SpawnTimer(Timer);

#[derive(Debug, Clone, Eq, PartialEq, Resource, Deref, DerefMut)]
struct Score(i32);

#[derive(Component)]
struct BoxEntity;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, plugin::ClickIndicatorPlugin))
        .insert_resource(SpawnTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
        .insert_resource(Score(0))
        .insert_resource(ClearColor(Color::srgba(0.1, 0.1, 0.1, 1.0)))
        .insert_resource(Score(0))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (spawn_boxes, animate_fragments).chain())
        .add_systems(Update, (update_scoreboard, click_to_destroy).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    // meshes: ResMut<Assets<Mesh>>,
    // materials: ResMut<Assets<ColorMaterial>>,
    // asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    commands
        // Root Node
        .spawn(NodeBundle {
            style: Style {
                width: Val::Vw(100.0),
                height: Val::Vw(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // テキスト
            parent.spawn((
                LabelUi,
                TextBundle::from_sections([
                    TextSection::new(
                        "Score:",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::srgba(0.5, 1.0, 0.5, 1.0),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "0",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::srgba(0.5, 1.0, 0.5, 1.0),
                            ..default()
                        },
                    ),
                ])
                .with_style(Style {
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Start,
                    ..default()
                }),
            ));
        });
}

fn update_scoreboard(label: Res<Score>, mut query: Query<&mut Text, With<LabelUi>>) {
    let mut ui = query.single_mut();
    ui.sections[1].value = label.to_string();
}

fn spawn_boxes(
    mut commands: Commands,
    mut timer: ResMut<SpawnTimer>,
    time: Res<Time>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    if timer.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let window = q_window.single();

        let width = window.width() / 2.0;
        let height = window.height() / 2.0;

        let x = rng.gen_range(-width..width);
        let y = rng.gen_range(-height..height);

        commands.spawn((
            BoxEntity,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.0..1.0),
                        rng.gen_range(0.5..1.0),
                    ),
                    custom_size: Some(Vec2::new(100.0, 100.0)),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(x, y, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
        ));
    }
}

/// クリックでボックスを破壊
fn click_to_destroy(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Sprite), With<BoxEntity>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut score: ResMut<Score>,
) {
    if !buttons.just_released(MouseButton::Left) {
        return;
    }

    let window = q_window.single();
    let Some(cursor_position) = window.cursor_position().map(|mut pos| {
        // 中心0,0の座標系に補正
        pos.x = pos.x - window.width() / 2.0;
        pos.y = window.height() / 2.0 - pos.y;
        pos
    }) else {
        return;
    };

    for (entity, transform, sprite) in query.iter_mut() {
        let box_position = transform.translation.truncate();
        // マウスカーソルがボックスの中心に近い場合に破壊判定する
        if (cursor_position - box_position).length() < sprite.custom_size.unwrap().length() / 2.0 {
            // ボックスを破壊
            commands.entity(entity).despawn();
            // スコア加算
            score.0 += 10;
            // 破片を生成
            spawn_fragments(&mut commands, sprite, cursor_position);
        }
    }
}

#[derive(Component)]
struct Fragment(Vec2, f32);

/// 破片を生成
fn spawn_fragments(commands: &mut Commands, sprite: &Sprite, position: Vec2) {
    let mut rng = rand::thread_rng();
    for _ in 0..15 {
        // クリック位置からのオフセット
        let offset_x = rng.gen_range(-10.0..10.0);
        let offset_y = rng.gen_range(-10.0..10.0);

        // 速度
        let velocity = Vec2::new(
            rng.gen_range(0.0..400.0) * if rng.gen_bool(0.5) { 1.0 } else { -1.0 },
            rng.gen_range(0.0..600.0),
        );
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::hsva(
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.5..1.0),
                    ),
                    // 1/1.5から1/5のサイズに縮小
                    custom_size: sprite
                        .custom_size
                        .map(|size| size / rng.gen_range(1.5..5.0)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(position.x + offset_x, position.y + offset_y, 0.0),
                ..Default::default()
            },
            Fragment(
                velocity,
                // Timer::from_seconds(5.0, TimerMode::Once),
                rng.gen_range(0.0..20.0),
            ),
        ));
    }
}

/// 破片のアニメーションを行う
fn animate_fragments(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Fragment, &mut Sprite)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();
    let window = q_window.single();
    for (entity, mut transform, mut fragment, mut sprite) in query.iter_mut() {
        if transform.scale.truncate().length() < 0.1 {
            // 破片の寿命が尽きたら削除
            commands.entity(entity).despawn();
        } else {
            // 重力を加算
            fragment.0.y += -400.0 * time.delta_seconds();

            // 速度を加算
            transform.translation.x += fragment.0.x * time.delta_seconds();
            transform.translation.y += fragment.0.y * time.delta_seconds();

            // Z軸を中心に回転
            transform.rotation *= Quat::from_rotation_z(fragment.1 * time.delta_seconds());

            // 色相を変更
            sprite.color = sprite.color.rotate_hue(rng.gen_range(0.0..10.0));

            let mut bound = false;
            // 画面外に出たらベクトルを反転かつ減衰
            if transform.translation.x.abs() > window.width() / 2.0 {
                fragment.0.x *= -0.96;
                bound = true
            }
            if transform.translation.y.abs() > window.height() / 2.0 {
                fragment.0.y *= -0.96;
                bound = true
            }

            // バウンドしたらサイズ縮小
            if bound {
                transform.scale = transform.scale * 0.7;
            }
        }
    }
}
