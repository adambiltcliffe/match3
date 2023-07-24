use std::f32::consts::PI;

use macroquad::prelude::*;

const GRID_W: usize = 10;
const GRID_H: usize = 10;

const TILE_W: f32 = 32.0;
const TILE_H: f32 = 32.0;

const SWAP_TIME: f32 = 0.15;
const SWAP_SWERVE: f32 = 8.0;

#[derive(Copy, Clone, PartialEq, Eq)]
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

#[derive(Copy, Clone)]
struct Tile {
    settled: bool,
    matched: bool,
    color: TileColor,
}

struct Swap {
    cx1: usize,
    cx2: usize,
    cy1: usize,
    cy2: usize,
    t: f32,
}

struct Fall {
    cx: usize,
    cy: usize,
    d: f32,
    v: f32,
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

fn make_board() -> [[Tile; GRID_H]; GRID_W] {
    let t = Tile {
        settled: true,
        matched: false,
        color: TileColor::Red,
    };
    let mut board = [[t; GRID_H]; GRID_W];
    for cx in 0..GRID_W {
        for cy in 0..GRID_H {
            board[cx][cy].color = TILE_COLORS[((cx * 3) + cy) % 7];
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
    let mut falls: Vec<Fall> = Vec::new();
    let mut check_matches = false;

    loop {
        if check_matches {
            let mut found_match = false;
            for cy in 0..GRID_H {
                let mut color: Option<TileColor> = None;
                let mut run = 0;
                for cx in 0..GRID_W {
                    if board[cx][cy].settled && color == Some(board[cx][cy].color) {
                        run += 1;
                    } else {
                        if run > 2 {
                            for dcx in 1..=run {
                                board[cx - dcx][cy].matched = true;
                            }
                            found_match = true;
                        }
                        if board[cx][cy].settled {
                            color = Some(board[cx][cy].color);
                            run = 1;
                        } else {
                            color = None;
                            run = 0;
                        }
                    }
                }
                if run > 2 {
                    for dcx in 1..=run {
                        board[GRID_W - dcx][cy].matched = true;
                    }
                    found_match = true;
                }
            }
            for cx in 0..GRID_W {
                let mut color: Option<TileColor> = None;
                let mut run = 0;
                for cy in 0..GRID_H {
                    if board[cx][cy].settled && color == Some(board[cx][cy].color) {
                        run += 1;
                    } else {
                        if run > 2 {
                            for dcy in 1..=run {
                                board[cx][cy - dcy].matched = true;
                            }
                            found_match = true;
                        }
                        if board[cx][cy].settled {
                            color = Some(board[cx][cy].color);
                            run = 1;
                        } else {
                            color = None;
                            run = 0;
                        }
                    }
                }
                if run > 2 {
                    for dcy in 1..=run {
                        board[cx][GRID_H - dcy].matched = true;
                    }
                    found_match = true;
                }
            }
            if found_match {
                for cx in 0..GRID_W {
                    let mut cy = GRID_H - 1;
                    let mut floor = GRID_H;
                    loop {
                        if board[cx][cy].matched {
                            if cy == 0 {
                                let d = floor as f32 * TILE_H;
                                for cy in (0..floor).rev() {
                                    board[cx][cy].color = random_color();
                                    board[cx][cy].matched = false;
                                    board[cx][cy].settled = false;
                                    falls.push(Fall { cx, cy, d, v: 0.0 });
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
                                board[cx][floor - 1].color = board[cx][cy].color;
                                board[cx][floor - 1].matched = false;
                                if !board[cx][floor - 1].settled {
                                    for ref mut f in &mut falls {
                                        if f.cx == cx && f.cy == cy {
                                            f.d += TILE_H * (floor - 1 - cy) as f32;
                                        }
                                    }
                                } else {
                                    board[cx][floor - 1].settled = false;
                                    falls.push(Fall {
                                        cx,
                                        cy: floor - 1,
                                        d: TILE_H * (floor - 1 - cy) as f32,
                                        v: 0.0,
                                    });
                                }
                                floor -= 1;
                                board[cx][cy].matched = true;
                            }
                        }
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
                        let tc = board[cx1][cy1].color;
                        board[cx1][cy1].color = board[cx2][cy2].color;
                        board[cx2][cy2].color = tc;
                        board[cx1][cy1].settled = false;
                        board[cx2][cy2].settled = false;
                        swap = Some(Swap {
                            cx1,
                            cy1,
                            cx2,
                            cy2,
                            t: 0.0,
                        });
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
                board[sw.cx1][sw.cy1].settled = true;
                board[sw.cx2][sw.cy2].settled = true;
                swap = None;
                check_matches = true;
            }
        }

        for ref mut f in &mut falls {
            if board[f.cx][f.cy].settled {
                println!("error error");
                f.d = 0.0;
            }
            f.d -= 2.5;
            if f.d <= 0.0 {
                board[f.cx][f.cy].settled = true;
                check_matches = true;
            }
        }
        falls.retain(|ref f| f.d > 0.0);

        clear_background(BLACK);
        for cx in 0..GRID_W {
            for cy in 0..GRID_H {
                let tx = cx as f32 * TILE_W;
                let ty = cy as f32 * TILE_H;
                if board[cx][cy].settled && !board[cx][cy].matched {
                    draw_single_tile(tx, ty, board[cx][cy].color);
                }
            }
        }

        for ref f in &falls {
            let tx = f.cx as f32 * TILE_W;
            let ty = f.cy as f32 * TILE_H - f.d;
            draw_single_tile(tx, ty, board[f.cx][f.cy].color);
        }

        if let Some(ref sw) = swap {
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
                board[sw.cx1][sw.cy1].color,
            );
            draw_single_tile(
                tx2 * t + tx1 * (1.0 - t) - dis_x,
                ty2 * t + ty1 * (1.0 - t) - dis_y,
                board[sw.cx2][sw.cy2].color,
            );
        }
        next_frame().await
    }
}
