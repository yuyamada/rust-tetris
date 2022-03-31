extern crate sdl2;

use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
// use sdl2::sys::StaticColor;
use sdl2::render::{Texture, TextureQuery, WindowCanvas};
use std::time::{Duration, Instant};
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const BLOCK_SIZE: u32 = 24;
const BLOCK_NUM_X: u32 = 10;
const BLOCK_NUM_Y: u32 = 20;
const STAGE_WIDTH: u32 = BLOCK_SIZE * BLOCK_NUM_X;
const STAGE_HEIGHT: u32 = BLOCK_SIZE * BLOCK_NUM_Y;
const STAGE_X: i32 = ((WIDTH - STAGE_WIDTH) / 2) as i32;
const STAGE_Y: i32 = ((HEIGHT - STAGE_HEIGHT) / 2) as i32;
const DANGER_LINE_Y: i32 = 5;
const TETRIMINO_MIN_X: i32 = 0;
const TETRIMINO_MAX_X: i32 = (STAGE_WIDTH / BLOCK_SIZE) as i32;
const TETRIMINO_DEFAULT_X: i32 = 3;
const TETRIMINO_MIN_Y: i32 = 0;
const TETRIMINO_MAX_Y: i32 = (STAGE_HEIGHT / BLOCK_SIZE) as i32;
const TETRIMINO_DEFAULT_Y: i32 = 0;
const GAME_OVER_X: i32 = -120;
const GAME_OVER_Y: i32 = 150;
const RETRY_MSG_X: i32 = -50;
const RETRY_MSG_Y: i32 = 270;

const FRAME_RATE: u32 = 60;
const FRAME_DURATION: Duration = Duration::new(0, 1000_000_000u32 / FRAME_RATE);
const MOVE_DOWN_FRAME_COUNT: u32 = 60;
const BASE_SCORE: u32 = 1000;

pub fn main() {
    let mut game = Game::new();
    let mut rng = rand::thread_rng();
    let mut shape: usize = rng.gen_range(0..game.tetriminoes.len());
    let mut rotate: usize = 0;
    let (mut tetrimino_x, mut tetrimino_y) = (TETRIMINO_DEFAULT_X, TETRIMINO_DEFAULT_Y);
    let mut frame_count: u32 = 0;
    let mut go_next = false;
    let mut score: u32 = 0;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem
        .window("Rust TETRIS", WIDTH, HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let font = ttf_context.load_font("assets/Mplus1-Bold.ttf", 32).unwrap();
    let surface = font.render("Rust TETRIS").blended(Color::WHITE).unwrap();
    let title_texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    let TextureQuery {
        width: title_width,
        height: title_height,
        ..
    } = title_texture.query();

    let font = ttf_context.load_font("assets/Mplus1-Bold.ttf", 80).unwrap();
    let surface = font.render("GAME OVER").blended(Color::RED).unwrap();
    let texture_creator = canvas.texture_creator();
    let game_over_texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    let TextureQuery {
        width: game_over_width,
        height: game_over_height,
        ..
    } = game_over_texture.query();

    let font = ttf_context.load_font("assets/Mplus1-Bold.ttf", 24).unwrap();
    let surface = font
        .render("Press RETURN Key To Retry")
        .blended(Color::RED)
        .unwrap();
    let texture_creator = canvas.texture_creator();
    let retry_msg_texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    let TextureQuery {
        width: retry_msg_width,
        height: retry_msg_height,
        ..
    } = retry_msg_texture.query();

    let mut event_pump = sdl_context.event_pump().unwrap();

    // main loop
    'running: loop {
        let start = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match game.state {
                    State::PLAYING => match keycode {
                        Keycode::Escape => break 'running,
                        Keycode::Left => {
                            move_tetrimino(
                                &game.stage,
                                &game.tetriminoes[shape][rotate],
                                &mut tetrimino_x,
                                &mut tetrimino_y,
                                -1,
                                0,
                            );
                        }
                        Keycode::Right => {
                            move_tetrimino(
                                &game.stage,
                                &game.tetriminoes[shape][rotate],
                                &mut tetrimino_x,
                                &mut tetrimino_y,
                                1,
                                0,
                            );
                        }
                        Keycode::Up => {
                            rotate_tetrimino(
                                &game.stage,
                                &game.tetriminoes[shape],
                                tetrimino_x,
                                tetrimino_y,
                                &mut rotate,
                            );
                        }
                        Keycode::Down => {
                            move_tetrimino(
                                &game.stage,
                                &game.tetriminoes[shape][rotate],
                                &mut tetrimino_x,
                                &mut tetrimino_y,
                                0,
                                1,
                            );
                        }
                        Keycode::Return => {
                            while move_tetrimino(
                                &game.stage,
                                &game.tetriminoes[shape][rotate],
                                &mut tetrimino_x,
                                &mut tetrimino_y,
                                0,
                                1,
                            ) {}
                            go_next = true;
                        }
                        _ => {}
                    },
                    State::GAMEOVER => match keycode {
                        Keycode::Escape => break 'running,
                        Keycode::Return => {
                            game = Game::new();
                            shape = rng.gen_range(0..game.tetriminoes.len());
                            rotate = 0;
                            tetrimino_x = TETRIMINO_DEFAULT_X;
                            tetrimino_y = TETRIMINO_DEFAULT_Y;
                            frame_count = 0;
                        }
                        _ => {}
                    },
                },
                _ => {}
            }
        }

        match game.state {
            State::PLAYING => {
                frame_count += 1;
                if frame_count == MOVE_DOWN_FRAME_COUNT {
                    frame_count = 0;
                    if !move_tetrimino(
                        &game.stage,
                        &game.tetriminoes[shape][rotate],
                        &mut tetrimino_x,
                        &mut tetrimino_y,
                        0,
                        1,
                    ) {
                        go_next = true;
                    }
                }

                if go_next {
                    go_next = false;
                    fix_tetrimino(
                        &mut game.stage,
                        &game.tetriminoes[shape][rotate],
                        tetrimino_x,
                        tetrimino_y,
                    );
                    let cleared_row_count = clear_blocks(&mut game.stage);
                    score += cleared_row_count.pow(2) * BASE_SCORE;
                    println!("score: {}", score);

                    if is_game_over(&game.stage) {
                        game.state = State::GAMEOVER;
                    } else {
                        shape = rng.gen_range(0..game.tetriminoes.len());
                        rotate = 0;
                        tetrimino_x = TETRIMINO_DEFAULT_X;
                        tetrimino_y = TETRIMINO_DEFAULT_Y;
                        frame_count = 0;
                    }
                }
            }
            _ => {}
        }

        render_background(&mut canvas);
        render_title(&mut canvas, &title_texture, title_width, title_height);
        let font = ttf_context.load_font("assets/Mplus1-Bold.ttf", 32).unwrap();
        let surface = font.render("Score").blended(Color::WHITE).unwrap();
        let score_texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let TextureQuery {
            width: score_width,
            height: score_height,
            ..
        } = title_texture.query();
        render_stage(&mut canvas, &game.stage);
        render_tetrimino(
            &mut canvas,
            &game.tetriminoes[shape][rotate],
            tetrimino_x,
            tetrimino_y,
        );
        render_danger_line(&mut canvas);

        match game.state {
            State::GAMEOVER => {
                render_game_over(
                    &mut canvas,
                    &game_over_texture,
                    game_over_width,
                    game_over_height,
                );
                render_retry_msg(
                    &mut canvas,
                    &retry_msg_texture,
                    retry_msg_width,
                    retry_msg_height,
                );
            }
            _ => {}
        }

        canvas.present();

        let end = Instant::now();
        let elapsed = end.duration_since(start);
        if elapsed < FRAME_DURATION {
            ::std::thread::sleep(FRAME_DURATION - elapsed)
        }
    }
}

struct Game {
    tetriminoes: Vec<Vec<Vec<Vec<u8>>>>,
    stage: Vec<Vec<u8>>,
    state: State,
}

impl Game {
    fn new() -> Game {
        // tetriminoes[shape][rotate][row][col].
        let tetriminoes = vec![
            // type-O
            vec![
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
            ],
            // type-I
            vec![
                vec![
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![1, 1, 1, 1, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 1, 1, 1],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
            ],
            // type-T
            vec![
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 1, 1, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 1, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 1, 1, 1, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
            ],
            // type-J
            vec![
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 1, 1, 0],
                    vec![0, 0, 0, 1, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 1, 1, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 0, 0, 0],
                    vec![0, 1, 1, 1, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
            ],
            // type-L
            vec![
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 1, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 1, 0],
                    vec![0, 1, 1, 1, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 1, 1, 0],
                    vec![0, 1, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
            ],
            // type-S
            vec![
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 1, 1, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 0, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 1, 1, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 1, 1, 0],
                    vec![0, 0, 0, 1, 0],
                    vec![0, 0, 0, 0, 0],
                ],
            ],
            // type-Z
            vec![
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 1, 1, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 1, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 0, 0],
                    vec![0, 1, 1, 0, 0],
                    vec![0, 0, 1, 1, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![
                    vec![0, 0, 0, 0, 0],
                    vec![0, 0, 0, 1, 0],
                    vec![0, 0, 1, 1, 0],
                    vec![0, 0, 1, 0, 0],
                    vec![0, 0, 0, 0, 0],
                ],
                vec![vec![0, 0, 0], vec![0, 1, 1], vec![1, 1, 0]],
                vec![vec![0, 1, 0], vec![0, 1, 1], vec![0, 0, 1]],
                vec![vec![0, 1, 1], vec![1, 1, 0], vec![0, 0, 0]],
                vec![vec![1, 0, 0], vec![1, 1, 0], vec![0, 1, 0]],
            ],
        ];
        // let stage = vec![vec![1; 10]; 20];
        let stage = vec![vec![0; 10]; 20];
        let state = State::PLAYING;
        Game {
            tetriminoes,
            stage,
            state,
        }
    }
}

enum State {
    PLAYING,
    GAMEOVER,
}

fn is_game_over(stage: &Vec<Vec<u8>>) -> bool {
    for i in 0..DANGER_LINE_Y as usize {
        for v in stage[i].iter() {
            if *v == 1 {
                return true;
            }
        }
    }
    false
}

fn rotate_tetrimino(
    stage: &Vec<Vec<u8>>,
    rotations: &Vec<Vec<Vec<u8>>>,
    x: i32,
    y: i32,
    rotate: &mut usize,
) -> bool {
    let new_rotate = ((*rotate + 1) % 4) as usize;
    if !is_movable(stage, &rotations[new_rotate], x, y, 0, 0) {
        return false;
    }
    *rotate = new_rotate;
    true
}

fn is_movable(
    stage: &Vec<Vec<u8>>,
    tetrimino: &Vec<Vec<u8>>,
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
) -> bool {
    for (i, row) in tetrimino.iter().enumerate() {
        for (j, v) in row.iter().enumerate() {
            if *v == 0 {
                continue;
            }
            let new_x = x + dx + j as i32;
            let new_y = y + dy + i as i32;

            // out of range
            if new_x < TETRIMINO_MIN_X || TETRIMINO_MAX_X <= new_x {
                return false;
            }
            if new_y < TETRIMINO_MIN_Y || TETRIMINO_MAX_Y <= new_y {
                return false;
            }

            // block exists
            if stage[new_y as usize][new_x as usize] == 1 {
                return false;
            }
        }
    }
    true
}

fn move_tetrimino(
    stage: &Vec<Vec<u8>>,
    tetrimino: &Vec<Vec<u8>>,
    x: &mut i32,
    y: &mut i32,
    dx: i32,
    dy: i32,
) -> bool {
    if !is_movable(stage, tetrimino, *x, *y, dx, dy) {
        return false;
    }
    *x += dx;
    *y += dy;
    true
}

// TODO: 範囲外になる場合の判定
fn fix_tetrimino(stage: &mut Vec<Vec<u8>>, tetrimino: &Vec<Vec<u8>>, x: i32, y: i32) {
    for (i, row) in tetrimino.iter().enumerate() {
        for (j, v) in row.iter().enumerate() {
            if *v == 0 {
                continue;
            }
            stage[(i as i32 + y) as usize][(j as i32 + x) as usize] = 1;
        }
    }
}

fn clear_blocks(stage: &mut Vec<Vec<u8>>) -> u32 {
    stage.retain(|row| row.iter().fold(0, |sum, a| sum + a) != row.len() as u8);
    let cleared_row_count: u32 = BLOCK_NUM_Y - stage.len() as u32;
    while stage.len() < BLOCK_NUM_Y as usize {
        stage.insert(0, vec![0; BLOCK_NUM_X as usize])
    }
    cleared_row_count
}

fn render_background(canvas: &mut WindowCanvas) {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
}

fn render_title(canvas: &mut WindowCanvas, texture: &Texture, width: u32, height: u32) {
    // render title
    canvas
        .copy(&texture, None, Some(Rect::new(20, 10, width, height)))
        .unwrap();
}

fn render_block(canvas: &mut WindowCanvas, x: i32, y: i32) {
    canvas.set_draw_color(Color::WHITE);
    canvas
        .fill_rect(Rect::new(
            STAGE_X + x * BLOCK_SIZE as i32,
            STAGE_Y + y * BLOCK_SIZE as i32,
            BLOCK_SIZE,
            BLOCK_SIZE,
        ))
        .unwrap();
}

fn render_blocks(canvas: &mut WindowCanvas, blocks: &Vec<Vec<u8>>, x: i32, y: i32) {
    for (i, row) in blocks.iter().enumerate() {
        for (j, v) in row.iter().enumerate() {
            if *v == 1 {
                render_block(canvas, x + j as i32, y + i as i32);
            }
        }
    }
}

fn render_stage(canvas: &mut WindowCanvas, stage: &Vec<Vec<u8>>) {
    canvas.set_draw_color(Color::BLUE);
    canvas
        .draw_rect(Rect::new(STAGE_X, STAGE_Y, STAGE_WIDTH, STAGE_HEIGHT))
        .unwrap();
    render_blocks(canvas, stage, 0, 0);
}

fn render_tetrimino(canvas: &mut WindowCanvas, tetrimino: &Vec<Vec<u8>>, x: i32, y: i32) {
    render_blocks(canvas, tetrimino, x, y);
}

fn render_danger_line(canvas: &mut WindowCanvas) {
    canvas.set_draw_color(Color::WHITE);
    canvas
        .draw_line(
            Point::new(STAGE_X, STAGE_Y + DANGER_LINE_Y * BLOCK_SIZE as i32),
            Point::new(
                STAGE_X + STAGE_WIDTH as i32,
                STAGE_Y + DANGER_LINE_Y * BLOCK_SIZE as i32,
            ),
        )
        .unwrap();
}

fn render_game_over(canvas: &mut WindowCanvas, texture: &Texture, width: u32, height: u32) {
    canvas
        .copy(
            &texture,
            None,
            Some(Rect::new(
                STAGE_X + GAME_OVER_X,
                STAGE_Y + GAME_OVER_Y,
                width,
                height,
            )),
        )
        .unwrap();
}

fn render_retry_msg(canvas: &mut WindowCanvas, texture: &Texture, width: u32, height: u32) {
    canvas
        .copy(
            &texture,
            None,
            Some(Rect::new(
                STAGE_X + RETRY_MSG_X,
                STAGE_Y + RETRY_MSG_Y,
                width,
                height,
            )),
        )
        .unwrap();
}
