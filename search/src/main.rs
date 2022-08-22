mod parser;
mod responder;

extern crate simple_server;
use simple_server::Server;
use simple_server::Request;
use simple_server::Response;
use simple_server::ResponseBuilder;

use std::time::Duration;

use url::Url;


use std::env;
use std::path::PathBuf;
use std::collections::{HashSet, HashMap};
use std::fs::File;
use std::io::prelude::*;   // write_all
use std::io::BufReader;
use std::path::Path;
use sudachi::prelude::MorphemeList;
use sudachi::config::Config;
use sudachi::analysis::Mode;
use sudachi::analysis::stateful_tokenizer::StatefulTokenizer;
use sudachi::dic::dictionary::JapaneseDictionary;

/*
fn get_dict() -> JapaneseDictionary {
    let config = Config::new(
	Some(PathBuf::from("./t/sudachi.rs/resources/sudachi.json")),
	Some(PathBuf::from("./t/sudachi.rs/resources")),
	Some(PathBuf::from("./t/sudachi.rs/resources/system.dic")),
    ).expect("Failed to load config file");
    JapaneseDictionary::from_cfg(&config).expect("Failed to read dict.")
}
*/

fn serve(request: Request<Vec<u8>>, mut response: ResponseBuilder) -> Response<Vec<u8>> {
    let uri = request.uri();
    let query_str = match uri.query() {
	Some(query) => query,
	None => return response.status(404).body("err".as_bytes().to_vec()).unwrap(),
    };
    let mut q = "".to_string();
    for (k, v) in url::form_urlencoded::parse(query_str.as_bytes()) {
	if k == "q" {
	    q = v.to_string();
	}
    }
    response.status(200).body(q.as_bytes().to_vec()).unwrap()
}

fn main() {
    let mut server = Server::new(|request, mut response| {
	Ok(serve(request, response))
    });
    server.dont_serve_static_files();
    server.listen("0.0.0.0", "9292");
}
