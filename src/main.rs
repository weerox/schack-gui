mod network;

use ggez::event;
use ggez::graphics::{self, DrawParam, DrawMode};
use ggez::{Context, GameResult};
use ggez::event::{MouseButton, KeyCode};
use std::{env};
use std::path;
use schackmotor::{Board, PieceType, Position};
use crate::network::NetworkHandler;
use std::sync::{Mutex, Arc};

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

struct InputHandler {
    clicked_tile: Option<Position>,
    clicked_tile_2: Option<Position>,
}

impl InputHandler {
    fn new() -> Self {
        InputHandler {
            clicked_tile: None,
            clicked_tile_2: None
        }
    }

    fn reset_clicked_squares(&mut self) {
        self.clicked_tile = None;
        self.clicked_tile_2 = None;
    }

    fn clicked_at(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32, data_handler: &mut DataHandler, graphics_handler: &mut GraphicsHandler) {
        if button == MouseButton::Left && self.clicked_tile_2.is_none() {
            let clicked_position: schackmotor::Position = GridPosition { x: (x / GRID_CELL_SIZE.0 as f32).floor() as i32, y: (y / GRID_CELL_SIZE.1 as f32).floor() as i32 }.into();

            if self.clicked_tile.is_none() {
                if let Some(moves) = data_handler.moves_from_position(clicked_position){
                    self.clicked_tile = Some(clicked_position);
                    for mov in moves {
                        graphics_handler.add_marked_tile(mov.0);
                    }
                }
            } else if data_handler.can_move_piece_at_position(clicked_position) {
                self.clicked_tile = Some(clicked_position);

                if let Some(moves) = data_handler.moves_from_position(clicked_position){
                    self.clicked_tile = Some(clicked_position);
                    graphics_handler.clear_marks();
                    for mov in moves {
                        graphics_handler.add_marked_tile(mov.0);
                    }
                }
            } else {
                let (moves, promotes) = data_handler.piece_at_position_can_move_to(self.clicked_tile.unwrap(), clicked_position);

                if moves {
                    if promotes {
                        self.clicked_tile_2 = Some(clicked_position);
                    } else {
                        self.forward_move(ctx, format!("{}-{}", self.clicked_tile.unwrap(), clicked_position), data_handler, graphics_handler);

                    }
                } else {
                    self.clicked_tile = None;
                    graphics_handler.clear_marks();
                }
            }
        }
    }

    fn key_pressed(&mut self, ctx: &mut Context, keycode: ggez::event::KeyCode, data_handler: &mut DataHandler, graphics_handler: &mut GraphicsHandler) {
        if self.clicked_tile_2.is_some() {
            match keycode {
                KeyCode::Q => {
                    self.forward_move(ctx, format!("{}-{}=Q", self.clicked_tile.unwrap(), self.clicked_tile_2.unwrap()), data_handler, graphics_handler);
                }
                KeyCode::R => {
                    self.forward_move(ctx, format!("{}-{}=R", self.clicked_tile.unwrap(), self.clicked_tile_2.unwrap()), data_handler, graphics_handler);
                }
                KeyCode::B => {
                    self.forward_move(ctx, format!("{}-{}=B", self.clicked_tile.unwrap(), self.clicked_tile_2.unwrap()), data_handler, graphics_handler);
                }
                KeyCode::N => {
                    self.forward_move(ctx, format!("{}-{}=N", self.clicked_tile.unwrap(), self.clicked_tile_2.unwrap()), data_handler, graphics_handler);
                }
                _ => {}
            }
        }
    }

    fn forward_move(&mut self, ctx: &mut Context, mov: String, data_handler: &mut DataHandler, graphics_handler: &mut GraphicsHandler) {
        data_handler.take_move(mov);
        graphics_handler.update_board(data_handler, ctx);
        self.reset_clicked_squares();
    }
}

struct DataHandler {
    board: Board,
    gameover: schackmotor::GameState,
}

impl DataHandler {
    fn new(board: Board) -> Self {
        DataHandler {
            board,
            gameover: schackmotor::GameState::Normal
        }
    }

    fn update_game_state(&mut self) {
        self.gameover = self.board.get_game_state();
    }

    fn take_move(&mut self, mov: String) {
        self.board.take_move(mov);
        self.update_game_state();

        //Transmit the move to the other client
    }

    fn moves_from_position(&self, position: schackmotor::Position) -> Option<Vec<(Position, bool)>> {
        self.board.get_possible_moves_from_position(position)
    }

    fn can_move_piece_at_position(&self, position: schackmotor::Position) -> bool {
        if self.board.get_piece_at(position).is_none() {
            return false;
        }
        self.board.get_piece_at(position).unwrap().get_color() == self.board.get_current_player()
    }

    fn piece_at_position_can_move_to(&self, start_position: schackmotor::Position, end_position: schackmotor::Position) -> (bool, bool) {
        if let Some(moves) = self.moves_from_position(start_position) {
            for mov in moves {
                if mov.0 == end_position {
                    return (true, mov.1);
                }
            }
        }
        (false, false)
    }
}

struct GraphicsHandler {
    sprites: Vec<((schackmotor::Color, schackmotor::PieceType), String)>,
    tiles: Vec<Tile>,
    graphics_pieces: Vec<GraphicsPiece>,
    marks: Vec<MarkedTile>,
}

impl GraphicsHandler {
    fn new(data_handler: &DataHandler, ctx: &mut Context) -> Self {
        let mut out = GraphicsHandler {
            sprites: GraphicsHandler::load_sprites(),
            tiles: Vec::new(),
            graphics_pieces: Vec::new(),
            marks: Vec::new()
        };

        out.populate_from_data(data_handler, ctx);

        out
    }

    fn populate_from_data(&mut self, data_handler: &DataHandler, ctx: &mut Context) {
        let mut tiles = Vec::new();
        for x in 0..8 {
            for y in 0..8 {
                tiles.push(Tile {
                    position: GridPosition { x, y },
                    color: if (x + y) % 2 == 0 { [1.0, 0.81, 0.62, 1.0].into() } else { [0.82, 0.55, 0.28, 1.0].into() },
                });
            }
        }

        self.tiles = tiles;

        self.update_board(data_handler, ctx);
    }

    fn update_board(&mut self, data_handler: &DataHandler, ctx: &mut Context) {
        let mut graphics_pieces = Vec::new();
        let pieces = data_handler.board.get_pieces();

        for piece in pieces {
            graphics_pieces.push(GraphicsPiece {
                sprite: graphics::Image::new(ctx, self.sprites.iter()
                    .find(|element| (element.0).0 == piece.get_color() && (element.0).1 == piece.get_type()).unwrap().1.clone()).unwrap(),
                position: GridPosition::from(piece.get_position()),
            });
        }

        self.graphics_pieces = graphics_pieces;

        self.marks.clear();
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

    fn draw(&mut self, gamestate: &schackmotor::GameState, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.5, 0.5, 0.5, 1.0].into());

        for tile in &self.tiles {
            tile.draw(ctx)?;
        }

        for graphics_piece in &self.graphics_pieces {
            graphics_piece.draw(ctx)?;
        }

        for mark in &self.marks {
            mark.draw(ctx)?;
        }

        let mut middle_of_screen_text = true;
        let mut text: String = "".to_string();
        match gamestate {
            schackmotor::GameState::Normal | schackmotor::GameState::Check(_) => {
                middle_of_screen_text = false
            }
            schackmotor::GameState::Checkmate(color) => {
                text = format!("{} has won", color);
            }
            schackmotor::GameState::Draw => {
                text = "Draw".to_string();
            }
        }

        if middle_of_screen_text {
            let gg_text = graphics::Text::new(graphics::TextFragment::from(text)
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

        Ok(())
    }

    fn add_marked_tile(&mut self, position: schackmotor::Position) {
        self.marks.push(MarkedTile::new(position));
    }

    fn clear_marks(&mut self) {
        self.marks.clear();
    }
}

struct GameState {
    data_handler: Arc<Mutex<DataHandler>>,
    graphics_handler: GraphicsHandler,
    input_handler: InputHandler,
    network_handler: NetworkHandler,
    last_network_update: u128
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let board = Board::new(Board::get_standard_layout());

        let data_handler = Arc::new(Mutex::new(DataHandler::new(board)));
        let graphics_handler = GraphicsHandler::new(&data_handler.lock().unwrap(), ctx);
        let network_handler = NetworkHandler::new("127.0.0.1:7878".to_string(), data_handler.clone());

        let mut state = GameState {
            data_handler,
            graphics_handler,
            input_handler: InputHandler::new(),
            network_handler,
            last_network_update: 0
        };

        Ok(state)
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {


        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.graphics_handler.draw(&self.data_handler.lock().unwrap().gameover, ctx)?;

        graphics::present(ctx)?;

        Ok(())
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        self.input_handler.clicked_at(ctx, button, x, y, &mut self.data_handler.lock().unwrap(), &mut self.graphics_handler);

    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: ggez::event::KeyCode, _keymod: ggez::event::KeyMods, _repeat: bool) {
        self.input_handler.key_pressed(ctx, keycode, &mut self.data_handler.lock().unwrap(), &mut self.graphics_handler);
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