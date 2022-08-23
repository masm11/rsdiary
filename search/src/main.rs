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

fn get_q(request: Request<Vec<u8>>) -> Option<String> {
    let uri = request.uri();
    let query_str = match uri.query() {
	Some(query) => query,
	None => return None,
    };
    let mut q = "".to_string();
    for (k, v) in url::form_urlencoded::parse(query_str.as_bytes()) {
	if k == "q" {
	    return Some(v.to_string());
	}
    }
    None
}

fn serve(request: Request<Vec<u8>>,
	 mut response: ResponseBuilder,
	 dict: &JapaneseDictionary,
	 index_words: &HashMap<String, u32>,
	 index_matrix: &HashMap<String, HashSet<u32>>) -> Response<Vec<u8>> {
    let q = match get_q(request) {
	Some(q) => q,
	None => return response.status(404).body("err".as_bytes().to_vec()).unwrap(),
    };

    let mut analyzer = StatefulTokenizer::new(dict, Mode::A);
    let mut parser = parser::Parser::new(&mut analyzer, index_words, index_matrix);
    let result = parser.parse(q.clone());

    let responder = responder::Responder::new();
    let html = responder.make_html(q, 1, result);

    response.status(200).body(html.as_bytes().to_vec()).unwrap()
}

fn index_file_path(typ: &str, suffix: &str) -> String {
    let mut path = env::var("INDEX_DIR").expect("Couldn't get INDEX_DIR");
    path.push_str("/");
    path.push_str("index.");
    path.push_str(typ);
    path.push_str(".txt");
    path.push_str(suffix);
    path
}

fn read_index_words() -> HashMap<String, u32> {
    let path = index_file_path("words", "");
    let path = Path::new(&path);
    let file = match File::open(&path) {
	Err(why) => panic!("couldn't open {}: {}", path.display(), why),
	Ok(file) => file,
    };
    let file = BufReader::new(file);

    let mut map = HashMap::<String, u32>::new();
    let mut word_id: u32 = 0;

    for line in file.lines() {
	let line = line.unwrap();
	map.insert(line, word_id);
	word_id += 1;
    }

    map
}

fn read_index_matrix() -> HashMap<String, HashSet<u32>> {
    let path = index_file_path("matrix", "");
    let path = Path::new(&path);
    let file = File::open(&path).expect("Failed to open index.matrix.txt.");
    let file = BufReader::new(file);

    let mut mat = HashMap::<String, HashSet<u32>>::new();

    for line in file.lines() {
	let line = line.unwrap();
	let mut iter = line.split_ascii_whitespace();
	let path = iter.next().unwrap();
	let mut word_ids = HashSet::<u32>::new();
	for s in iter {
	    let id: u32 = s.parse().unwrap();
	    word_ids.insert(id);
	}
	mat.insert(path.to_string(), word_ids);
    }

    mat
}

fn get_dict() -> JapaneseDictionary {
    let config = Config::new(
	Some(PathBuf::from("../t/sudachi.rs/resources/sudachi.json")),
	Some(PathBuf::from("../t/sudachi.rs/resources")),
	Some(PathBuf::from("../t/sudachi.rs/resources/system.dic")),
    ).expect("Failed to load config file");
    JapaneseDictionary::from_cfg(&config).expect("Failed to read dict.")
}

fn main() {
    let dict = get_dict();
    let index_words = read_index_words();
    let index_matrix = read_index_matrix();
    let mut server = Server::new(move |request, mut response| {
	Ok(serve(request, response, &dict, &index_words, &index_matrix))
    });
    server.dont_serve_static_files();
    server.listen("0.0.0.0", "9292");
}
