use ggez::event;
use ggez::graphics::{self, Color, DrawParam, DrawMode, Drawable};
use ggez::{Context, GameResult};
use std::env;
use std::path;
use schackmotor::{Board, PieceType, Position};
use ggez::event::{MouseButton, KeyCode};
use ggez::nalgebra::{Vector2, Point2};

const GRID_SIZE: (i16, i16) = (8, 8);
const GRID_CELL_SIZE: (i16, i16) = (45, 45);

const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GridPosition {
    x: i32,
    y: i32,
}

impl GridPosition {
    fn from(pos: schackmotor::Position) -> Self {
        GridPosition {
            x: pos.get_x() as i32 - 1,
            y: (8 - pos.get_y()) as i32,
        }
    }

    fn to_rect(&self) -> graphics::Rect {
        graphics::Rect::new_i32(
            self.x * GRID_CELL_SIZE.0 as i32,
            self.y * GRID_CELL_SIZE.1 as i32,
            GRID_CELL_SIZE.0 as i32,
            GRID_CELL_SIZE.1 as i32,
        )
    }
}

impl From<GridPosition> for graphics::Rect {
    fn from(pos: GridPosition) -> Self {
        graphics::Rect::new_i32(
            pos.x * GRID_CELL_SIZE.0 as i32,
            pos.y * GRID_CELL_SIZE.1 as i32,
            GRID_CELL_SIZE.0 as i32,
            GRID_CELL_SIZE.1 as i32,
        )
    }
}

impl From<GridPosition> for ggez::mint::Point2<f32> {
    fn from(pos: GridPosition) -> Self {
        ggez::mint::Point2 { x: (pos.x * GRID_CELL_SIZE.0 as i32) as f32, y: (pos.y * GRID_CELL_SIZE.1 as i32) as f32 }
    }
}

impl From<GridPosition> for schackmotor::Position {
    fn from(pos: GridPosition) -> Self {
        schackmotor::Position::new((pos.x + 1) as u8, (8 - pos.y) as u8)
    }
}

struct Tile {
    position: GridPosition,
    color: graphics::Color,
}

impl Tile {
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let rectangle = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), self.position.into(), self.color)?;
        graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 }, ))
    }
}

struct GraphicsPiece {
    sprite: graphics::Image,
    position: GridPosition,
}

impl GraphicsPiece {
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        graphics::draw(ctx, &self.sprite, DrawParam::default().dest(self.position)
            .scale(ggez::mint::Vector2 { x: GRID_CELL_SIZE.0 as f32 / 45.0, y: GRID_CELL_SIZE.1 as f32 / 45.0 }))
    }
}

struct MarkedTile {
    position: GridPosition
}

impl MarkedTile {
    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let circle = graphics::Mesh::new_circle
            (ctx, DrawMode::fill(), graphics::mint::Point2 { x: 22.5, y: 22.5 }, 7.5, 1.0, [0.5, 0.5, 0.5, 0.5].into())?;
        graphics::draw(ctx, &circle, DrawParam::default().dest(self.position))
    }

    fn new(position: Position) -> MarkedTile {
        MarkedTile { position: GridPosition::from(position) }
    }
}

struct GameState {
    sprites: Vec<((schackmotor::Color, schackmotor::PieceType), String)>,
    tiles: Vec<Tile>,
    graphics_pieces: Vec<GraphicsPiece>,
    marks: Vec<MarkedTile>,
    board: Board,
    clicked_tile: Option<Position>,
    clicked_tile_2: Option<Position>,
    gameover: Option<schackmotor::Color>,
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let sprites = GameState::load_sprites();

        let mut tiles = Vec::new();
        for x in 0..8 {
            for y in 0..8 {
                tiles.push(Tile {
                    position: GridPosition { x, y },
                    color: if (x + y) % 2 == 0 { [1.0, 0.81, 0.62, 1.0].into() } else { [0.82, 0.55, 0.28, 1.0].into() },
                });
            }
        }

        let board = Board::new(Board::get_standard_layout());
        let mut graphics_pieces = Vec::new();
        let pieces = board.get_pieces();

        for piece in pieces {
            graphics_pieces.push(GraphicsPiece {
                sprite: graphics::Image::new(ctx, sprites.iter()
                    .find(|element| (element.0).0 == piece.get_color() && (element.0).1 == piece.get_type()).unwrap().1.clone())?,
                position: GridPosition::from(piece.get_position()),
            });
        }

        let marks = Vec::new();

        let mut state = GameState {
            sprites,
            tiles,
            graphics_pieces,
            board,
            marks,
            clicked_tile: None,
            clicked_tile_2: None,
            gameover: None,
        };

        Ok(state)
    }

    fn update_board(&mut self, ctx: &mut Context) {
        let mut graphics_pieces = Vec::new();
        let pieces = self.board.get_pieces();

        for piece in pieces {
            graphics_pieces.push(GraphicsPiece {
                sprite: graphics::Image::new(ctx, self.sprites.iter()
                    .find(|element| (element.0).0 == piece.get_color() && (element.0).1 == piece.get_type()).unwrap().1.clone()).unwrap(),
                position: GridPosition::from(piece.get_position()),
            });
        }

        self.graphics_pieces = graphics_pieces;

        self.marks.clear();
        self.clicked_tile = None;
        self.clicked_tile_2 = None;
    }

    fn load_sprites() -> Vec<((schackmotor::Color, schackmotor::PieceType), String)> {
        let mut sprites = Vec::new();
        sprites.push(((schackmotor::Color::Black, PieceType::King), "/black_king.png".to_string()));
        sprites.push(((schackmotor::Color::Black, PieceType::Queen), "/black_queen.png".to_string()));
        sprites.push(((schackmotor::Color::Black, PieceType::Rook), "/black_rook.png".to_string()));
        sprites.push(((schackmotor::Color::Black, PieceType::Pawn), "/black_pawn.png".to_string()));
        sprites.push(((schackmotor::Color::Black, PieceType::Bishop), "/black_bishop.png".to_string()));
        sprites.push(((schackmotor::Color::Black, PieceType::Knight), "/black_knight.png".to_string()));
        sprites.push(((schackmotor::Color::White, PieceType::King), "/white_king.png".to_string()));
        sprites.push(((schackmotor::Color::White, PieceType::Queen), "/white_queen.png".to_string()));
        sprites.push(((schackmotor::Color::White, PieceType::Rook), "/white_rook.png".to_string()));
        sprites.push(((schackmotor::Color::White, PieceType::Pawn), "/white_pawn.png".to_string()));
        sprites.push(((schackmotor::Color::White, PieceType::Bishop), "/white_bishop.png".to_string()));
        sprites.push(((schackmotor::Color::White, PieceType::Knight), "/white_knight.png".to_string()));
        sprites
    }

    fn check_for_gameover(&mut self, ctx: &mut Context) {
        if self.board.get_current_player_moves().iter().map(|element| element.1.len())
            .fold(0, |acc, element| acc + element) == 0 {
            self.gameover = Some(self.board.get_current_player().invert());
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.5, 0.5, 0.5, 1.0].into());

        for tile in &self.tiles {
            tile.draw(ctx);
        }

        for graphics_piece in &self.graphics_pieces {
            graphics_piece.draw(ctx);
        }

        for mark in &self.marks {
            mark.draw(ctx);
        }

        if self.gameover.is_some() {
            let gg_text = graphics::Text::new(graphics::TextFragment::from(format!("{} has won", self.gameover.unwrap()))
                .scale(graphics::Scale { x: 45.0, y: 45.0 }));
            let gg_dimensions = gg_text.dimensions(ctx);
            let background_box = graphics::Mesh::new_rectangle(ctx, DrawMode::fill(),
                                                               graphics::Rect::new((SCREEN_SIZE.0 - gg_dimensions.0 as f32) / 2f32 as f32 - 8.0,
                                                                                   (SCREEN_SIZE.0 - gg_dimensions.1 as f32) / 2f32 as f32,
                                                                                   gg_dimensions.0 as f32 + 16.0, gg_dimensions.1 as f32),
                                                               [1.0, 1.0, 1.0, 1.0].into())?;
            graphics::draw(ctx, &background_box, DrawParam::default());
            graphics::draw(ctx, &gg_text, DrawParam::default().color([0.0, 0.0, 0.0, 1.0].into())
                .dest(ggez::mint::Point2 {
                    x: (SCREEN_SIZE.0 - gg_dimensions.0 as f32) / 2f32 as f32,
                    y: (SCREEN_SIZE.0 - gg_dimensions.1 as f32) / 2f32 as f32,
                }));
        }

        graphics::present(ctx)?;

        Ok(())
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left && self.clicked_tile_2.is_none() {
            let clicked_position: schackmotor::Position = GridPosition { x: (x / GRID_CELL_SIZE.0 as f32).floor() as i32, y: (y / GRID_CELL_SIZE.1 as f32).floor() as i32 }.into();

            if self.clicked_tile.is_none() {
                for piece in self.board.get_current_player_moves() {
                    if piece.0.get_position() == clicked_position {
                        self.clicked_tile = Some(clicked_position);
                        for moves in piece.1 {
                            self.marks.push(MarkedTile::new(moves.0));
                        }
                    }
                }
            } else if self.board.get_pieces().iter()
                .find(|piece| piece.get_color() == self.board.get_current_player() && piece.get_position() == clicked_position)
                .is_some() {
                self.clicked_tile = Some(clicked_position);

                for piece in self.board.get_current_player_moves() {
                    if piece.0.get_position() == clicked_position {
                        self.clicked_tile = Some(clicked_position);
                        self.marks.clear();
                        for moves in piece.1 {
                            self.marks.push(MarkedTile::new(moves.0));
                        }
                    }
                }
            } else {
                let moves = self.board.get_current_player_moves();
                let mut maybe_move: Option<String> = None;
                let mut no_move = true;
                for mov in moves {
                    if mov.0.get_position() == self.clicked_tile.unwrap() {
                        for tup in mov.1 {
                            if tup.0 == clicked_position && tup.1 && mov.0.get_type().is_pawn() {
                                self.clicked_tile_2 = Some(clicked_position);
                                no_move = false;
                            } else if tup.0 == clicked_position && !tup.1 {
                                maybe_move = Some(format!("{}-{}", self.clicked_tile.unwrap(), clicked_position));
                                no_move = false;
                            }
                        }
                    }
                }
                if maybe_move.is_some() {
                    self.board.take_move(maybe_move.unwrap());
                    self.update_board(ctx);
                    self.check_for_gameover(ctx);
                }
                if no_move {
                    self.clicked_tile = None;
                    self.marks.clear();
                }
            }
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: ggez::event::KeyCode, _keymod: ggez::event::KeyMods, repeat: bool) {
        if self.clicked_tile_2.is_some() {
            match keycode {
                KeyCode::Q => {
                    println!("{:?}, {:?}", self.clicked_tile, self.clicked_tile_2);
                    self.board.take_move(format!("{}-{}=Q", self.clicked_tile.unwrap(), self.clicked_tile_2.unwrap()));
                    self.update_board(ctx);
                    self.check_for_gameover(ctx);
                }
                KeyCode::R => {
                    self.board.take_move(format!("{}-{}=R", self.clicked_tile.unwrap(), self.clicked_tile_2.unwrap()));
                    self.update_board(ctx);
                    self.check_for_gameover(ctx);
                }
                KeyCode::B => {
                    self.board.take_move(format!("{}-{}=B", self.clicked_tile.unwrap(), self.clicked_tile_2.unwrap()));
                    self.update_board(ctx);
                    self.check_for_gameover(ctx);
                }
                KeyCode::N => {
                    self.board.take_move(format!("{}-{}=N", self.clicked_tile.unwrap(), self.clicked_tile_2.unwrap()));
                    self.update_board(ctx);
                    self.check_for_gameover(ctx);
                }
                _ => {}
            }
        }
        if self.board.get_current_player_moves().len() == 0 {
            println!("ggez {}", self.board.get_current_player());
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

    let cb = ggez::ContextBuilder::new("schack", "eskil").add_resource_path(resource_dir)
        .window_setup(ggez::conf::WindowSetup::default().title("Schack").icon("/icon.ico"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1).resizable(false));
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut GameState::new(ctx)?;
    event::run(ctx, event_loop, state)
}