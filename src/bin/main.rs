use raylib::prelude::*;

#[derive(Clone, Copy)]
pub enum State {
    Dead,
    Alive,
}

impl State {
    pub fn get_next_state(&self, neighbor_count: usize) -> Self {
        match (self, neighbor_count) {
            (Self::Alive, 2 | 3) => Self::Alive,
            (Self::Dead, 3) => Self::Alive,
            (_, _) => Self::Dead,
        }
    }
}

fn wrapping_bound(value: usize, offset: isize, width: usize) -> usize {
    assert!(offset.abs() < width as isize);
    match usize::try_from(offset) {
        Ok(positive_offset) => (value + positive_offset) % width,
        Err(_) => (value + width - (-offset as usize)) % width,
    }
}

fn main() {
    const WIDTH: usize = 80;
    const HEIGHT: usize = 40;
    const SCALE: usize = 20;
    const PADDING: usize = 1;

    let mut board = [[State::Dead; WIDTH]; HEIGHT];
    board[0][1] = State::Alive;
    board[1][2] = State::Alive;
    board[2][2] = State::Alive;
    board[2][1] = State::Alive;
    board[2][0] = State::Alive;

    let (mut rl, thread) = raylib::init()
        .size((WIDTH * SCALE) as _, (HEIGHT * SCALE) as _)
        .title("Game of Life")
        .build();

    let mut ticks_per_second = 5.0;
    let mut is_paused = true;
    let mut time = 0.0;
    let mut highlight_start: Option<(usize, usize)> = None;
    let mut clipboard: Vec<Vec<State>> = vec![];
    while !rl.window_should_close() {
        let ts = rl.get_frame_time();

        let mouse_pos = rl.get_mouse_position() / SCALE as f32;
        let (x, y) = (mouse_pos.x as isize, mouse_pos.y as isize);

        // drawing
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::DARKGRAY);
            for (y, row) in board.iter().enumerate() {
                for (x, state) in row.iter().enumerate() {
                    d.draw_rectangle(
                        (x * SCALE + PADDING) as _,
                        (y * SCALE + PADDING) as _,
                        (SCALE - PADDING * 2) as _,
                        (SCALE - PADDING * 2) as _,
                        match state {
                            State::Dead => Color::BLACK,
                            State::Alive => Color::WHITE,
                        },
                    );
                }
            }

            if let Some((start_x, start_y)) = highlight_start {
                d.draw_rectangle_lines_ex(
                    Rectangle {
                        x: (start_x * SCALE) as f32,
                        y: (start_y * SCALE) as f32,
                        width: (x as f32 - start_x as f32 + 1.0) * SCALE as f32,
                        height: (y as f32 - start_y as f32 + 1.0) * SCALE as f32,
                    },
                    5,
                    Color::BLUE,
                );
            }

            if is_paused {
                let length = raylib::text::measure_text("PAUSED", 30);
                d.draw_text(
                    "PAUSED",
                    (WIDTH * SCALE / 2) as i32 - length / 2,
                    (HEIGHT * SCALE / 2) as _,
                    30,
                    Color::WHITE,
                );
            }
            d.draw_text(
                &format!("Ticks Per Second: {ticks_per_second:.0}"),
                5,
                5,
                20,
                Color::RED,
            );
        }

        // input
        {
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                is_paused = !is_paused;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_W) {
                ticks_per_second *= 2.0;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_S) {
                ticks_per_second /= 2.0;
            }
            if is_paused {
                if x >= 0 && x < WIDTH as isize && y >= 0 && y < HEIGHT as isize {
                    if rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                        if rl.is_mouse_button_pressed(MouseButton::MOUSE_RIGHT_BUTTON) {
                            for (y_offset, row) in clipboard.iter().enumerate() {
                                for (x_offset, state) in row.iter().enumerate() {
                                    let new_x = wrapping_bound(x as _, x_offset as _, WIDTH);
                                    let new_y = wrapping_bound(y as _, y_offset as _, HEIGHT);
                                    board[new_y][new_x] = *state;
                                }
                            }
                        }
                    } else {
                        if rl.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
                            let state = &mut board[y as usize][x as usize];
                            *state = match state {
                                State::Dead => State::Alive,
                                State::Alive => State::Dead,
                            };
                        }
                        if rl.is_mouse_button_pressed(MouseButton::MOUSE_RIGHT_BUTTON) {
                            highlight_start = Some((x as _, y as _));
                        }
                        if rl.is_mouse_button_released(MouseButton::MOUSE_RIGHT_BUTTON) {
                            let (start_x, start_y) = highlight_start.unwrap();
                            clipboard = vec![
                                vec![State::Dead; x as usize - start_x + 1];
                                y as usize - start_y + 1
                            ];
                            for y in start_y..=y as _ {
                                for x in start_x..=x as _ {
                                    clipboard[y - start_y][x - start_x] = board[y][x];
                                }
                            }
                            highlight_start = None;
                        }
                    }
                }
            }
        }

        // update
        if !is_paused {
            time += ts;
            while time >= 1.0 / ticks_per_second {
                let mut neighbors = [[0usize; WIDTH]; HEIGHT];
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        if let State::Alive = board[y][x] {
                            for y_offset in -1..=1isize {
                                for x_offset in -1..=1isize {
                                    if x_offset != 0 || y_offset != 0 {
                                        let new_x = wrapping_bound(x, x_offset, WIDTH);
                                        let new_y = wrapping_bound(y, y_offset, HEIGHT);
                                        neighbors[new_y][new_x] += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                board = std::array::from_fn(|y| {
                    std::array::from_fn(|x| board[y][x].get_next_state(neighbors[y][x]))
                });
                time -= 1.0 / ticks_per_second;
            }
        }
    }
}
