use ggez;
use nalgebra;

use ggez::event;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use nalgebra as na;
use std::env;
use std::path;
use schackmotor::{Board, PieceType};
use ggez::event::MouseButton;

struct GameState {
    text: graphics::Text,
    canvas: graphics::Canvas,
    frames: usize,
    draw_with_canvas: bool,
    chess_board: Board
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let chess_board = Board::new(Board::get_standard_layout());
        let font = graphics::Font::new(ctx, "/STD.ttf")?;
        let text = graphics::Text::new(("RrNnBbQqKkPp\n123456789\n!\"-#$%&'.()*+,S", font, 48.0));
        let canvas = graphics::Canvas::with_window_size(ctx)?;

        graphics::set_drawable_size(ctx, 800 as f32, 600 as f32);
        graphics::set_resizable(ctx, false);
        graphics::set_window_title(ctx, "Schack");

        let mut s = GameState {
            text,
            canvas,
            draw_with_canvas: false,
            frames: 0,
            chess_board: Board::new(Board::get_standard_layout())
        };

        s.text = graphics::Text::new((s.get_board_as_string().iter().map(|row| row.iter()
            .fold("".to_string(), |acc, element| format!("{}{}", acc, element)))
                                          .fold("".to_string(), |acc, element| format!("{}{}\n", acc, element)), font, 48.0));

        Ok(s)
    }

    fn get_board_as_string(&self) -> Vec<Vec<String>> {
        let mut out: Vec<Vec<String>> = Vec::new();

        for y in 0..10 {
            out.push(Vec::new());
            for x in 0..10 {
                out[y].push("".to_string());
            }
        }

        let pieces = self.chess_board.get_pieces();

        for x in 1..9 {
            for y in 1..9 {
                out[y][x] = if (x + y) % 2 == 0 { " ".to_string() } else { "0".to_string() }
            }
        }

        out[0][0] = "7".to_string(); //Top left corner
        out[9][0] = "1".to_string(); //Bottom left corner
        out[0][9] = "9".to_string(); //Top right corner
        out[9][9] = "3".to_string(); //Bottom right corner

        for range in 1..9 {
            out[0][range] = "8".to_string();
            out[range][0] = "4".to_string();
            out[9][range] = "2".to_string();
            out[range][9] = "6".to_string();
        }

        for piece in pieces {
            println!("{}", (piece.get_position().get_x() + piece.get_position().get_y()) % 2);
            out[piece.get_position().get_y() as usize][piece.get_position().get_x() as usize] = (if (piece.get_position().get_x() + piece.get_position().get_y()) % 2 == 0 {
                match piece.get_color() {
                    schackmotor::Color::White => {
                        match piece.get_type() {
                            PieceType::Rook => {"R"}
                            PieceType::King => {"K"}
                            PieceType::Queen => {"Q"}
                            PieceType::Bishop => {"B"}
                            PieceType::Knight => {"N"}
                            PieceType::Pawn => {"P"}
                        }
                    }
                    schackmotor::Color::Black => {
                        match piece.get_type() {
                            PieceType::Rook => {"r"}
                            PieceType::King => {"k"}
                            PieceType::Queen => {"q"}
                            PieceType::Bishop => {"b"}
                            PieceType::Knight => {"n"}
                            PieceType::Pawn => {"p"}
                        }
                    }
                }
            } else {
                match piece.get_color() {
                    schackmotor::Color::White => {
                        match piece.get_type() {
                            PieceType::Rook => {","}
                            PieceType::King => {"("}
                            PieceType::Queen => {"+"}
                            PieceType::Bishop => {"."}
                            PieceType::Knight => {")"}
                            PieceType::Pawn => {"*"}
                        }
                    }
                    schackmotor::Color::Black => {
                        match piece.get_type() {
                            PieceType::Rook => {"&"}
                            PieceType::King => {"-"}
                            PieceType::Queen => {"!"}
                            PieceType::Bishop => {"!"}
                            PieceType::Knight => {"#"}
                            PieceType::Pawn => {"$"}
                        }
                    }
                }
            }).to_string();
        }

        return out;
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let dest_point = na::Point2::new(10.0, 10.0);

        if self.draw_with_canvas {
            //println!("Drawing with canvas");
            graphics::clear(ctx, graphics::Color::from((64, 0, 0, 0)));

            graphics::set_canvas(ctx, Some(&self.canvas));
            graphics::clear(ctx, graphics::Color::from((255, 255, 255, 128)));

            graphics::draw(
                ctx,
                &self.text,
                graphics::DrawParam::new()
                    .dest(dest_point)
                    .color(Color::from((0, 0, 0, 255))),
            )?;
            graphics::set_canvas(ctx, None);

            // graphics::draw(ctx, &self.canvas, na::Point2::new(0.0, 0.0), 0.0)?;

            graphics::draw(
                ctx,
                &self.canvas,
                graphics::DrawParam::new().color(Color::from((255, 255, 255, 128))),
            )?;
        } else {
            //println!("Drawing without canvas");
            graphics::set_canvas(ctx, None);
            graphics::clear(ctx, [0.25, 0.0, 0.0, 1.0].into());

            graphics::draw(
                ctx,
                &self.text,
                graphics::DrawParam::new()
                    .dest(dest_point)
                    .color(Color::from((192, 128, 64, 255))),
            )?;
        }

        graphics::present(ctx)?;

        self.frames += 1;
        if (self.frames % 100) == 0 {
            println!("FPS: {}", ggez::timer::fps(ctx));
        }

        Ok(())
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            println!("Clicked on tile {}, {}", ((((x - 55f32)/43f32).floor() + 97f32) as u8) as char, 8f32 - ((y-60f32)/48f32).floor());

        }
        println!("Mouse button released: {:?}, x: {}, y: {}", button, x, y);
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        _keycode: ggez::event::KeyCode,
        _keymod: ggez::event::KeyMods,
        repeat: bool,
    ) {
        if !repeat {
            self.draw_with_canvas = !self.draw_with_canvas;
            println!("Canvas on: {}", self.draw_with_canvas);
        }
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("hello_canvas", "ggez").add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut GameState::new(ctx)?;
    event::run(ctx, event_loop, state)
}