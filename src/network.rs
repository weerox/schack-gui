use std::thread;
use crate::DataHandler;
use std::sync::{Mutex, Arc};

pub(crate) struct NetworkHandler {
    target_address: String,
    local_color: schackmotor::Color,
    data_handler: Arc<Mutex<DataHandler>>,
}

impl NetworkHandler {
    pub(crate) fn new(target_address: String, data_handler: Arc<Mutex<DataHandler>>) -> Self {
        let mut out = NetworkHandler {
            target_address,
            local_color: schackmotor::Color::White,
            data_handler,
        };

        out.listen();

        out
    }

    fn listen(&mut self) {
        let data_handler2 = self.data_handler.clone();

        thread::spawn(move || {
            let server = tiny_http::Server::http("0.0.0.0:7878").unwrap();

            loop {
                let mut request = match server.recv() {
                    Ok(rq) => rq,
                    Err(e) => { panic!("error: {}", e); }
                };

                let mut request_text = "".to_string();

                println!("{}", request.as_reader().read_to_string(&mut request_text).unwrap());

                println!("Method: {}", request.method());
                println!("Request: {}", request_text);

                match request.method() {
                    tiny_http::Method::Get => {
                        if request_text.contains("\"requested_data\":\"gamestate\"") {
                            let mut response =
                                tiny_http::Response::from_string(
                                    format!("{0}\"state\":\"{2}\"{1}", "{", "}",
                                            match data_handler2.lock().unwrap().gameover {
                                                schackmotor::GameState::Draw => { "draw" }
                                                schackmotor::GameState::Checkmate(color) => {
                                                    if color == schackmotor::Color::Black {
                                                        "black-won"
                                                    } else {
                                                        "white-won"
                                                    }
                                                }
                                                _ => { "in-progress" }
                                            }
                                    ));
                            request.respond(response);
                        } else if request_text.contains("\"requested_data\":\"current-turn\"") {
                            let text = format!("{0}\"current_turn\":\"{2}\"{1}", "{", "}",
                                               match data_handler2.lock().unwrap().board.get_current_player() {
                                                   schackmotor::Color::White => { "white" }
                                                   schackmotor::Color::Black => { "black" }
                                               }
                            );

                            let mut response = tiny_http::Response::from_string(text);
                            request.respond(response);
                        } else if request_text.contains("\"requested_data\":\"board\"") {
                            let mutex_guard = data_handler2.lock().unwrap();
                            let pieces = mutex_guard.board.get_pieces();
                            let mut inner_string = pieces.iter().map(|piece|
                                format!("{0}\"piece-type\":\"{2}\",\"color\":\"{3}\",\"position\":\"{4}\"{1}", "{", "}",
                                        match piece.get_type() {
                                            schackmotor::PieceType::Pawn => { "pawn" }
                                            schackmotor::PieceType::King => { "king" }
                                            schackmotor::PieceType::Queen => { "queen" }
                                            schackmotor::PieceType::Rook => { "rook" }
                                            schackmotor::PieceType::Bishop => { "bishop" }
                                            schackmotor::PieceType::Knight => { "knight" }
                                        }, match piece.get_color() {
                                        schackmotor::Color::White => { "white" }
                                        schackmotor::Color::Black => { "black" }
                                    }, piece.get_position()));

                            let mut total_string = inner_string.next().unwrap();
                            for elem in inner_string {
                                total_string = format!("{},{}", total_string, elem);
                            }

                            let pieces_string = format!("{0}\"board\":[{2}]{1}", "{", "}", total_string);

                            let mut response =
                                tiny_http::Response::from_string(pieces_string);
                            request.respond(response);
                        }
                    }
                    tiny_http::Method::Post => {}
                    _ => {
                        request.respond(tiny_http::Response::from_string("prutt"));
                    }
                }
            }
        });
    }
}