use std::time::Duration;

use bevy::{
    app::{App, Plugin, Update},
    color::Color,
    input::ButtonInput,
    math::Vec2,
    prelude::{
        Commands, Component, Entity, IntoSystemConfigs, MouseButton, Query, Res, Transform, With,
    },
    sprite::{Sprite, SpriteBundle},
    time::{Time, Timer, TimerMode},
    window::{PrimaryWindow, Window},
};

pub struct ClickIndicatorPlugin;

impl Plugin for ClickIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_indicator_on_click, remove_indicator_on_timeout).chain(),
        );
    }
}

// コンポーネント: 丸点の寿命を管理
#[derive(Component)]
struct Indicator(Timer);

fn spawn_indicator_on_click(
    mut commands: Commands,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    let window = q_window.single();
    let Some(position) = window.cursor_position().map(|mut pos| {
        pos.x = pos.x - window.width() / 2.0;
        pos.y = window.height() / 2.0 - pos.y;
        pos
    }) else {
        return;
    };

    if !mouse_input.just_released(MouseButton::Left) {
        return;
    }

    commands.spawn((
        Indicator(Timer::new(Duration::from_secs(1), TimerMode::Once)),
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.1, 0.9, 0.1, 1.0),  // 色
                custom_size: Some(Vec2::new(10.0, 10.0)), // サイズ
                ..Default::default()
            },
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            ..Default::default()
        },
    ));
}

fn remove_indicator_on_timeout(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Indicator)>,
    time: Res<Time>,
) {
    for (entity, mut indicator) in query.iter_mut() {
        if indicator.0.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
