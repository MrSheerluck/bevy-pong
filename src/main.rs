use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::window::WindowResolution;

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const PADDLE_WIDTH: f32 = 10.0;
const PADDLE_HEIGHT: f32 = 100.0;
const PADDLE_SPEED: f32 = 500.0;
const PADDLE_OFFSET: f32 = 50.0;
const BALL_SIZE: f32 = 10.0;

#[derive(Component)]
struct Paddle {
    speed: f32,
    side: Side,
}

#[derive(Component)]
enum Side {
    Left,
    Right,
}

#[derive(Component)]
struct Ball {
    velocity: Vec3,
}

#[derive(Resource, Default)]
struct Score {
    left: u32,
    right: u32,
}

#[derive(Component)]
struct ScoreText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pong".into(),
                resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_paddle,
                move_ball,
                bounce_ball,
                check_paddle_collision,
                score_goal,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.insert_resource(Score::default());

    commands.spawn((
        Text2d::new("0 - 0"),
        TextFont {
            font_size: FontSize::Px(40.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, WINDOW_HEIGHT / 2.0 - 50.0, 0.0),
        ScoreText,
    ));

    // Left paddle
    commands.spawn((
        Paddle {
            speed: PADDLE_SPEED,
            side: Side::Left,
        },
        Sprite::from_color(Color::WHITE, Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
        Transform::from_xyz(-WINDOW_WIDTH / 2.0 + PADDLE_OFFSET, 0.0, 0.0),
    ));

    // Right paddle
    commands.spawn((
        Paddle {
            speed: PADDLE_SPEED,
            side: Side::Right,
        },
        Sprite::from_color(Color::WHITE, Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
        Transform::from_xyz(WINDOW_WIDTH / 2.0 - PADDLE_OFFSET, 0.0, 0.0),
    ));

    // Ball
    commands.spawn((
        Ball {
            velocity: Vec3::new(300.0, 150.0, 0.0),
        },
        Sprite::from_color(Color::WHITE, Vec2::splat(BALL_SIZE)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn move_paddle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut paddle_query: Query<(&mut Transform, &Paddle)>,
    time: Res<Time>,
) {
    for (mut transform, paddle) in &mut paddle_query {
        let mut direction = 0.0;

        match paddle.side {
            Side::Left => {
                if keyboard.pressed(KeyCode::KeyW) {
                    direction = 1.0;
                }
                if keyboard.pressed(KeyCode::KeyS) {
                    direction = -1.0;
                }
            }
            Side::Right => {
                if keyboard.pressed(KeyCode::ArrowUp) {
                    direction = 1.0;
                }
                if keyboard.pressed(KeyCode::ArrowDown) {
                    direction = -1.0;
                }
            }
        }

        transform.translation.y += direction * paddle.speed * time.delta_secs();

        let half_paddle = PADDLE_HEIGHT / 2.0;
        let half_height = WINDOW_HEIGHT / 2.0;
        transform.translation.y = transform
            .translation
            .y
            .clamp(-half_height + half_paddle, half_height - half_paddle);
    }
}

fn move_ball(mut ball_query: Query<(&mut Transform, &Ball)>, time: Res<Time>) {
    for (mut transform, ball) in &mut ball_query {
        transform.translation += ball.velocity * time.delta_secs();
    }
}

fn bounce_ball(mut ball_query: Query<(&mut Transform, &mut Ball)>) {
    let half_height = WINDOW_HEIGHT / 2.0;
    let half_ball = BALL_SIZE / 2.0;

    for (mut transform, mut ball) in &mut ball_query {
        if transform.translation.y + half_ball >= half_height {
            transform.translation.y = half_height - half_ball;
            ball.velocity.y = -ball.velocity.y;
        }
        if transform.translation.y - half_ball <= -half_height {
            transform.translation.y = -half_height + half_ball;
            ball.velocity.y = -ball.velocity.y;
        }
    }
}

fn check_paddle_collision(
    mut ball_query: Query<(&mut Transform, &mut Ball), Without<Paddle>>,
    paddle_query: Query<(&Transform, &Paddle), Without<Ball>>,
) {
    for (mut ball_transform, mut ball) in &mut ball_query {
        let ball_pos = ball_transform.translation.truncate();
        let ball_size = Vec2::splat(BALL_SIZE);

        for (paddle_transform, _paddle) in &paddle_query {
            let paddle_pos = paddle_transform.translation.truncate();
            let paddle_size = Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT);

            let overlap = !(ball_pos.x + ball_size.x / 2.0 < paddle_pos.x - paddle_size.x / 2.0
                || ball_pos.x - ball_size.x / 2.0 > paddle_pos.x + paddle_size.x / 2.0
                || ball_pos.y + ball_size.y / 2.0 < paddle_pos.y - paddle_size.y / 2.0
                || ball_pos.y - ball_size.y / 2.0 > paddle_pos.y + paddle_size.y / 2.0);

            if overlap {
                ball.velocity.x = -ball.velocity.x;

                if ball.velocity.x > 0.0 {
                    ball_transform.translation.x =
                        paddle_pos.x + paddle_size.x / 2.0 + ball_size.x / 2.0;
                } else {
                    ball_transform.translation.x =
                        paddle_pos.x - paddle_size.x / 2.0 - ball_size.x / 2.0;
                }
            }
        }
    }
}

fn score_goal(
    mut ball_query: Query<&mut Transform, With<Ball>>,
    mut score: ResMut<Score>,
    mut score_text: Query<&mut Text2d, With<ScoreText>>,
) {
    let half_width = WINDOW_WIDTH / 2.0;

    for mut transform in &mut ball_query {
        let scored = if transform.translation.x > half_width + 10.0 {
            score.left += 1;
            true
        } else if transform.translation.x < -half_width - 10.0 {
            score.right += 1;
            true
        } else {
            false
        };

        if !scored {
            continue;
        }

        transform.translation = Vec3::new(0.0, 0.0, 0.0);

        for mut text in &mut score_text {
            text.0 = format!("{} - {}", score.left, score.right);
        }
    }
}
