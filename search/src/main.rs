mod parser;

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

fn get_dict() -> JapaneseDictionary {
    let config = Config::new(
	Some(PathBuf::from("./t/sudachi.rs/resources/sudachi.json")),
	Some(PathBuf::from("./t/sudachi.rs/resources")),
	Some(PathBuf::from("./t/sudachi.rs/resources/system.dic")),
    ).expect("Failed to load config file");
    JapaneseDictionary::from_cfg(&config).expect("Failed to read dict.")
}

fn tokenize(string: String, dict: &JapaneseDictionary) -> HashSet<String> {
    let mut set = HashSet::<String>::new();

    let mut analyzers = [
	StatefulTokenizer::new(dict, Mode::A),
	StatefulTokenizer::new(dict, Mode::B),
	StatefulTokenizer::new(dict, Mode::C),
    ];
    for ana in analyzers.iter_mut() {
	ana.reset().push_str(&string[..]);
	ana.do_tokenize().expect("Failed to tokenize.");
	let mut morphs = MorphemeList::empty(ana.dict_clone());
	morphs.collect_results(ana).expect("Failed to collect results.");
	for m in morphs.iter() {
	    set.insert(m.surface().to_string());
	    set.insert(m.normalized_form().to_string());
	}
    }
    set
}

fn read_index_words() -> HashMap<String, u32> {
    let path = Path::new("index.words.txt");
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
    let path = Path::new("index.matrix.txt");
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

fn main() {
    let dict = get_dict();

    let index_words = read_index_words();
    let index_matrix = read_index_matrix();

    let args: Vec<String> = env::args().collect();
    let query = args[1].clone();

    let set = tokenize(query, &dict);
    
    let mut ids = HashSet::<u32>::new();
    for word in set.iter() {
	let id: u32 = match index_words.get(word) {
	    Some(id) => *id,
	    None => panic!("Unknown word: {}", word),
	};
	ids.insert(id);
    }

    for (fname, word_ids) in index_matrix {
	if word_ids.is_superset(&ids) {
	    println!("{}", fname)
	}
    }

    let parser = crate::parser::Parser::new();
    parser.parse(String::from(""));
}
