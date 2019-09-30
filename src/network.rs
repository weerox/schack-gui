use std::thread;
use crate::{DataHandler, NotatedMove};
use std::sync::{Mutex, Arc};
use regex::Regex;
use std::io::Read;

pub(crate) struct NetworkHandler {
    client: reqwest::Client,
    target_address: String,
    local_color: Arc<Mutex<Option<schackmotor::Color>>>,
    data_handler: Arc<Mutex<DataHandler>>,
    score: (usize, usize), //number of times white has won, number of times black has won
    draw_requested: Arc<Mutex<(bool, bool)>>, //you, the guy she tells you not to worry about/your opponent
    rematch_requested: Arc<Mutex<(bool, bool)>> //you, the guy she tells you not to worry about/your opponent
}

impl NetworkHandler {
    pub(crate) fn get_target_address(&self) -> String {
        self.target_address.clone()
    }

    pub(crate) fn new(target_address: String, data_handler: Arc<Mutex<DataHandler>>) -> Self {
        let mut out = NetworkHandler {
            client: reqwest::Client::new(),
            target_address,
            local_color: Arc::new(Mutex::new(None)),
            data_handler,
            score: (0, 0),
            draw_requested: Arc::new(Mutex::new((false, false))),
            rematch_requested: Arc::new(Mutex::new((false, false)))
        };

        out.listen();

        out
    }

    fn listen(&mut self) {
        let data_handler2 = self.data_handler.clone();
        let local_color_ref = self.local_color.clone();
        let request_draw_ref = self.draw_requested.clone();
        let request_rematch_ref = self.rematch_requested.clone();
        let address = self.target_address.clone();

        thread::spawn( move || {
            let server = tiny_http::Server::http("0.0.0.0:7878").unwrap();
            let regex_for_start_square = Regex::new("\"start_square\"(\\s)*:(\\s)*\"[a-h][1-8]\"").unwrap();
            let regex_for_end_square = Regex::new("\"end_square\"(\\s)*:(\\s)*\"[a-h][1-8]\"").unwrap();
            let regex_for_promotion = Regex::new("\"promote_to\"(\\s)*:(\\s)*\"[QRBN]\"").unwrap();
            let regex_for_square_extraction = Regex::new("[a-h][1-8]").unwrap();
            let regex_for_promotion_extraction = Regex::new("[QRBN]").unwrap();

            loop {
                let mut request = match server.recv() {
                    Ok(rq) => rq,
                    Err(e) => { panic!("error: {}", e); }
                };

                let mut request_text = "".to_string();

                request.as_reader().read_to_string(&mut request_text).unwrap();

                println!("{}", request_text);

                match request.method() {
                    tiny_http::Method::Get => {}
                    tiny_http::Method::Post => {
                        let url = request.url();
                        let mut response_body = "".to_string();
                        let mut response_code = 0u32;

                        if url == "/start-game" {
                            if local_color_ref.lock().unwrap().is_none() {
                                if request_text.contains("white") {
                                    *local_color_ref.lock().unwrap() = Some(schackmotor::Color::Black);
                                    response_body = "{\"accepted\":true}".to_string();
                                    response_code = 200;
                                } else if request_text.contains("black") {
                                    *local_color_ref.lock().unwrap() = Some(schackmotor::Color::White);
                                    response_body = "{\"accepted\":true}".to_string();
                                    response_code = 200;
                                } else {
                                    response_body = "{\"accepted\":false}".to_string();
                                    response_code = 400;
                                }
                            }
                        } else if url == "/move" {
                            if regex_for_start_square.is_match(request_text.as_ref())
                                && regex_for_end_square.is_match(request_text.as_ref()) {
                                let start_square = regex_for_square_extraction.captures(
                                    regex_for_start_square.captures(request_text.as_ref()).unwrap()
                                        .get(0).unwrap().as_str()).unwrap().get(0).unwrap().as_str();
                                let end_square = regex_for_square_extraction.captures(
                                    regex_for_end_square.captures(request_text.as_ref()).unwrap()
                                        .get(0).unwrap().as_str()).unwrap().get(0).unwrap().as_str();
                                let res;
                                if regex_for_promotion.is_match(request_text.as_ref()) {
                                    let promotes_to = regex_for_promotion_extraction.captures(
                                        regex_for_promotion.captures(request_text.as_ref())
                                            .unwrap().get(0).unwrap().as_str()).unwrap()
                                        .get(0).unwrap().as_str();
                                    res = data_handler2.lock().unwrap().receive_move(
                                        NotatedMove::new(start_square.to_string(), end_square.to_string(),
                                                         Some(promotes_to.to_string())));
                                } else {
                                    res = data_handler2.lock().unwrap().receive_move(
                                        NotatedMove::new(start_square.to_string(), end_square.to_string(), None));
                                }

                                match res {
                                    Ok(_) => {
                                        response_body = "{\"valid_move\":true}".to_string();
                                        response_code = 200;
                                    }
                                    Err(_) => {
                                        response_body = "{\"valid_move\":false}".to_string();
                                        response_code = 400;
                                    }
                                }
                            }
                        } else if url == "/request-draw" {
                            request_draw_ref.lock().unwrap().1 = true;
                            response_body = format!("{0}\"draw_accepted\":{2}{1}", "{", "}", request_draw_ref.lock().unwrap().0);
                            response_code = 200;
                        } else if url == "/request-rematch" {
                            request_rematch_ref.lock().unwrap().1 = true;
                            response_body = format!("{0}\"draw_accepted\":{2}{1}", "{", "}", request_rematch_ref.lock().unwrap().0);
                            response_code = 200;
                        }

                        let mut response = tiny_http::Response::from_string(response_body);
                        response = response.with_status_code(tiny_http::StatusCode::from(response_code));

                        request.respond(response).unwrap();
                    }
                    _ => {
                        let mut res = tiny_http::Response::from_string("");
                        res = res.with_status_code(tiny_http::StatusCode::from(405));
                        request.respond(res).unwrap();
                    }
                }
            }
        });
    }

    pub(crate) fn send(&self, url: &str, message: String) -> String {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::HOST, reqwest::header::HeaderValue::from_bytes(url.as_ref()).unwrap());
        headers.insert(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_bytes(b"text/json").unwrap());
        let mut var = self.client.post(url).headers(headers).body(message).send().unwrap();

        let mut text = "".to_string();
        var.read_to_string(&mut text);

        println!("{}", text);

        text
    }
}