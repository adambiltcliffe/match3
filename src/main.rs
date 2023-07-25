use std::f32::consts::PI;

use macroquad::prelude::*;

const GRID_W: usize = 10;
const GRID_H: usize = 10;

const TILE_W: f32 = 32.0;
const TILE_H: f32 = 32.0;

const SWAP_TIME: f32 = 0.15;
const SWAP_SWERVE: f32 = 8.0;
const GRAVITY: f32 = 200.0;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TileColor {
    Red = 0,
    Orange = 1,
    Yellow = 2,
    Green = 3,
    Cyan = 4,
    Blue = 5,
    Magenta = 6,
}

const TILE_COLORS: [TileColor; 7] = [
    TileColor::Red,
    TileColor::Orange,
    TileColor::Yellow,
    TileColor::Green,
    TileColor::Cyan,
    TileColor::Blue,
    TileColor::Magenta,
];
const DRAW_COLORS: [Color; 7] = [RED, ORANGE, YELLOW, GREEN, SKYBLUE, BLUE, MAGENTA];

#[derive(Copy, Clone, Debug)]
enum TileState {
    JustMatched(TileColor),
    Settled(TileColor),
    Swapping(TileColor),
    Falling { color: TileColor, d: f32, v: f32 },
}

impl TileState {
    fn color(&self) -> TileColor {
        match self {
            TileState::JustMatched(c) => *c,
            TileState::Settled(c) => *c,
            TileState::Swapping(c) => *c,
            TileState::Falling { color, .. } => *color,
        }
    }

    fn matchable_color(&self) -> Option<TileColor> {
        match self {
            TileState::JustMatched(c) => Some(*c),
            TileState::Settled(c) => Some(*c),
            _ => None,
        }
    }
}

struct Swap {
    cx1: usize,
    cx2: usize,
    cy1: usize,
    cy2: usize,
    t: f32,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Match-3 game".to_owned(),
        fullscreen: false,
        window_width: GRID_W as i32 * TILE_W as i32,
        window_height: GRID_H as i32 * TILE_H as i32,
        ..Default::default()
    }
}

fn random_color() -> TileColor {
    TILE_COLORS[macroquad::rand::gen_range(0, 2)]
}

fn make_board() -> [[TileState; GRID_H]; GRID_W] {
    let t = TileState::Settled(TileColor::Red);
    let mut board = [[t; GRID_H]; GRID_W];
    for cx in 0..GRID_W {
        for cy in 0..GRID_H {
            board[cx][cy] = TileState::Settled(TILE_COLORS[((cx * 3) + cy) % 7]);
        }
    }
    board
}

fn get_mouse_cell() -> (usize, usize) {
    let (px, py) = mouse_position();
    ((px / TILE_W) as usize, (py / TILE_H) as usize)
}

fn draw_single_tile(px: f32, py: f32, c: TileColor) {
    draw_rectangle(px, py, TILE_W, TILE_H, BLACK);
    draw_rectangle(
        px + 1.0,
        py + 1.0,
        TILE_W - 2.0,
        TILE_H - 2.0,
        DRAW_COLORS[c as usize],
    );
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut board = make_board();
    let mut drag: Option<(usize, usize)> = None;
    let mut swap: Option<Swap> = None;
    let mut check_matches = false;

    loop {
        if check_matches {
            let mut found_match = false;
            for cy in 0..GRID_H {
                let mut color: Option<TileColor> = None;
                let mut run = 0;
                for cx in 0..GRID_W {
                    if color.is_some() && board[cx][cy].matchable_color() == color {
                        run += 1;
                    } else {
                        if run > 2 {
                            for dcx in 1..=run {
                                // pop!
                                assert!(board[cx - dcx][cy].matchable_color().is_some());
                                board[cx - dcx][cy] =
                                    TileState::JustMatched(board[cx - dcx][cy].color());
                            }
                            found_match = true;
                        }
                        color = board[cx][cy].matchable_color();
                        run = if color.is_some() { 1 } else { 0 };
                    }
                }
                if run > 2 {
                    for dcx in 1..=run {
                        // pop!
                        assert!(board[GRID_W - dcx][cy].matchable_color().is_some());
                        board[GRID_W - dcx][cy] =
                            TileState::JustMatched(board[GRID_W - dcx][cy].color());
                    }
                    found_match = true;
                }
            }
            for cx in 0..GRID_W {
                let mut color: Option<TileColor> = None;
                let mut run = 0;
                for cy in 0..GRID_H {
                    if color.is_some() && board[cx][cy].matchable_color() == color {
                        run += 1
                    } else {
                        if run > 2 {
                            for dcy in 1..=run {
                                // pop!
                                assert!(board[cx][cy - dcy].matchable_color().is_some());
                                board[cx][cy - dcy] =
                                    TileState::JustMatched(board[cx][cy - dcy].color());
                            }
                            found_match = true;
                        }
                        color = board[cx][cy].matchable_color();
                        run = if color.is_some() { 1 } else { 0 };
                    }
                }
                if run > 2 {
                    for dcy in 1..=run {
                        // pop!
                        assert!(board[cx][GRID_H - dcy].matchable_color().is_some());
                        board[cx][GRID_H - dcy] =
                            TileState::JustMatched(board[cx][GRID_H - dcy].color());
                    }
                    found_match = true;
                }
            }
            if found_match {
                for cx in 0..GRID_W {
                    let was_ok_before = validate(board[cx]);
                    let starting_state = board[cx].clone();
                    drop_column(&mut board[cx]);
                    let is_ok_now = validate(board[cx]);
                    if was_ok_before && !is_ok_now {
                        println!("WE BROKE IT");
                        println!("BEFORE: {:?}", starting_state);
                        println!("AFTER: {:?}", board[cx]);
                    }
                }
            }
        }
        check_matches = false;

        let can_act = swap.is_none();
        if drag.is_none() && is_mouse_button_pressed(MouseButton::Left) {
            drag = Some(get_mouse_cell());
        }
        if drag.is_some() && is_mouse_button_released(MouseButton::Left) {
            if can_act {
                if let Some((cx1, cy1)) = drag {
                    let (cx2, cy2) = get_mouse_cell();
                    if cx1.abs_diff(cx2) + cy1.abs_diff(cy2) == 1 {
                        if let (TileState::Settled(c1), TileState::Settled(c2)) =
                            (board[cx1][cy1], board[cx2][cy2])
                        {
                            board[cx1][cy1] = TileState::Swapping(c2);
                            board[cx2][cy2] = TileState::Swapping(c1);
                            swap = Some(Swap {
                                cx1,
                                cy1,
                                cx2,
                                cy2,
                                t: 0.0,
                            });
                        } else {
                            unreachable!();
                        }
                    }
                }
            }
            drag = None;
        }

        // update animations
        let delta = get_frame_time();
        if let Some(ref mut sw) = swap {
            sw.t += delta;
            if sw.t > SWAP_TIME {
                if let (TileState::Swapping(c1), TileState::Swapping(c2)) =
                    (board[sw.cx1][sw.cy1], board[sw.cx2][sw.cy2])
                {
                    board[sw.cx1][sw.cy1] = TileState::Settled(c1);
                    board[sw.cx2][sw.cy2] = TileState::Settled(c2);
                    swap = None;
                    check_matches = true;
                } else {
                    unreachable!()
                }
            }
        }

        let mut falls = 0;
        for cx in 0..GRID_W {
            for cy in (0..GRID_H).rev() {
                if let TileState::Falling { color, d, v } = board[cx][cy] {
                    let mut d = d - v * delta - 0.5 * GRAVITY * delta.powf(2.0);
                    let mut v = v + delta * GRAVITY;
                    if cy < GRID_H - 1 {
                        if let TileState::Falling { d: od, v: ov, .. } = board[cx][cy + 1] {
                            d = d.max(od);
                            v = v.min(ov);
                        }
                    }
                    if d <= 0.0 {
                        board[cx][cy] = TileState::Settled(color);
                        check_matches = true;
                    } else {
                        board[cx][cy] = TileState::Falling { color, d, v };
                        falls += 1;
                    }
                }
            }
        }

        clear_background(BLACK);
        for cx in 0..GRID_W {
            for cy in 0..GRID_H {
                let tx = cx as f32 * TILE_W;
                let ty = cy as f32 * TILE_H;
                if let TileState::Settled(color) = board[cx][cy] {
                    draw_single_tile(tx, ty, color);
                } else if let TileState::Falling { color, d, v } = board[cx][cy] {
                    draw_single_tile(tx, ty - d, color);
                }
            }
        }

        if let Some(ref sw) = swap {
            if let (TileState::Swapping(c1), TileState::Swapping(c2)) =
                (board[sw.cx1][sw.cy1], board[sw.cx2][sw.cy2])
            {
                let tx1 = sw.cx1 as f32 * TILE_W;
                let ty1 = sw.cy1 as f32 * TILE_H;
                let tx2 = sw.cx2 as f32 * TILE_W;
                let ty2 = sw.cy2 as f32 * TILE_H;
                let t = sw.t / SWAP_TIME;
                let dis_x = (t * PI).sin() * (ty2 - ty1) / TILE_H * SWAP_SWERVE;
                let dis_y = (t * PI).sin() * (tx2 - tx1) / TILE_W * SWAP_SWERVE;
                draw_single_tile(
                    tx1 * t + tx2 * (1.0 - t) + dis_x,
                    ty1 * t + ty2 * (1.0 - t) + dis_y,
                    c1,
                );
                draw_single_tile(
                    tx2 * t + tx1 * (1.0 - t) - dis_x,
                    ty2 * t + ty1 * (1.0 - t) - dis_y,
                    c2,
                );
            } else {
                unreachable!()
            }
        }
        next_frame().await
    }
}

fn drop_column(column: &mut [TileState; GRID_H]) {
    let mut cy = GRID_H - 1;
    let mut floor = GRID_H;
    loop {
        if let TileState::JustMatched(_) = column[cy] {
            if cy == 0 {
                let d = if floor == GRID_H {
                    floor as f32 * TILE_H
                } else {
                    match column[floor] {
                        TileState::Settled(_) => floor as f32 * TILE_H,
                        TileState::Falling { d, .. } => (floor as f32 * TILE_H).max(d),
                        _ => unreachable!(),
                    }
                };
                for cy in (0..floor).rev() {
                    column[cy] = TileState::Falling {
                        color: random_color(),
                        d,
                        v: 0.0,
                    };
                }
                break;
            }
            cy -= 1;
        } else {
            if floor == cy + 1 {
                floor = cy;
                if cy == 0 {
                    break;
                }
                cy -= 1;
            } else {
                let d = match column[cy] {
                    TileState::Settled(_) => TILE_H * (floor - 1 - cy) as f32,
                    TileState::Falling { d, .. } => d + TILE_H * (floor - 1 - cy) as f32,
                    _ => unreachable!(),
                };
                let v = match column[cy] {
                    TileState::Settled(_) => 0.0,
                    TileState::Falling { v, .. } => v,
                    _ => unreachable!(),
                };
                let color = match column[cy] {
                    TileState::Settled(c) => c,
                    TileState::Falling { color, .. } => color,
                    _ => unreachable!(),
                };
                column[floor - 1] = TileState::Falling { color, d, v };
                floor -= 1;
                column[cy] = TileState::JustMatched(TileColor::Red);
            }
        }
    }
}

fn validate(column: [TileState; GRID_H]) -> bool {
    let mut d = 0.0;
    for cy in (0..GRID_H).rev() {
        let new_d = match column[cy] {
            TileState::Settled(_) => 0.0,
            TileState::Falling { d, .. } => d,
            TileState::JustMatched(_) => continue,
            _ => unreachable!(),
        };
        if new_d < d {
            println!("bad result: {:?}", column);
            return false;
        }
        d = new_d;
    }
    true
}

#[test]
fn regression1() {
    let mut col = [
        TileState::Falling {
            color: TileColor::Magenta,
            d: 63.5,
            v: 0.0,
        },
        TileState::Falling {
            color: TileColor::Blue,
            d: 63.5,
            v: 0.0,
        },
        TileState::JustMatched(TileColor::Red),
        TileState::JustMatched(TileColor::Red),
        TileState::JustMatched(TileColor::Red),
        TileState::Settled(TileColor::Red),
        TileState::Settled(TileColor::Orange),
        TileState::Settled(TileColor::Yellow),
        TileState::Settled(TileColor::Green),
        TileState::Settled(TileColor::Red),
    ];
    drop_column(&mut col);
    assert!(validate(col));
}

#[test]
fn regression2() {
    let mut col = [
        TileState::Settled(TileColor::Magenta),
        TileState::JustMatched(TileColor::Red),
        TileState::Settled(TileColor::Green),
        TileState::Settled(TileColor::Orange),
        TileState::Settled(TileColor::Yellow),
        TileState::Settled(TileColor::Red),
        TileState::Settled(TileColor::Red),
        TileState::Settled(TileColor::Red),
        TileState::Settled(TileColor::Red),
        TileState::Settled(TileColor::Red),
    ];
    drop_column(&mut col);
    println!("{:?}", col);
    assert!(validate(col));
    assert!(matches!(col[0], TileState::Falling { d, .. } if d == 32.0));
    assert!(matches!(col[1], TileState::Falling { d, .. } if d == 32.0));
}
