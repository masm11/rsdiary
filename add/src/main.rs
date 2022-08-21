use std::env;
use std::path::PathBuf;
use std::collections::{HashSet, HashMap};
use std::fs;
use std::fs::File;
use std::io::prelude::*;   // write_all
use std::io::BufReader;
use std::path::Path;
use sudachi::prelude::MorphemeList;
use sudachi::config::Config;
use sudachi::analysis::Mode;
use sudachi::analysis::stateful_tokenizer::StatefulTokenizer;
use sudachi::dic::dictionary::JapaneseDictionary;

fn replace_lf(buf: &String) -> String {
    buf.replace("\n", " ")
	.replace("\r", " ")
	.replace("\t", " ")
}

fn get_dict() -> JapaneseDictionary {
    let res_dir0 = env::var("RES_DIR").expect("Couldn't get RES_DIR");
    let res_dir1 = env::var("RES_DIR").expect("Couldn't get RES_DIR");
    let res_dir2 = env::var("RES_DIR").expect("Couldn't get RES_DIR");
    let mut json = PathBuf::from(res_dir0);
    json.push("sudachi.json");
    let mut sys_dic = PathBuf::from(res_dir2);
    sys_dic.push("system.dic");
    let config = Config::new(
	Some(json),
	Some(PathBuf::from(res_dir1)),
	Some(sys_dic),
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

fn write_index_words(words: HashMap<String, u32>) {
    let path = index_file_path("words", ".new");
    let path = Path::new(&path);
    let mut file = File::create(&path).expect("Failed to create index.words.txt.new.");

    let mut max_id = 0;
    for (_, id) in words.iter() {
	if max_id < *id {
	    max_id = *id;
	}
    }
    let empty_string = String::from("");
    let mut ary: Vec<&String> = vec![&empty_string; (max_id + 1) as usize];
    for (s, id) in words.iter() {
	ary[*id as usize] = s;
    }

    for s in ary.iter() {
	file.write_all(s.as_bytes()).expect("Failed to write word.");
	file.write_all(b"\n").expect("Failed to write LF.");
    }
}

fn write_index_matrix(mat: HashMap::<String, HashSet<u32>>) {
    let path = index_file_path("matrix", ".new");
    let path = Path::new(&path);
    let mut file = File::create(&path).expect("Failed to create index.matrix.txt.new.");

    for (fname, word_ids) in mat.iter() {
	file.write_all(fname.as_bytes()).expect("Failed to write filename.");
	let mut delim = b"\t";
	for id in word_ids.iter() {
	    file.write_all(delim).expect("Failed to write delimiter.");
	    file.write_all(id.to_string().as_bytes()).expect("Failed to write word_id.");
	    delim = b" ";
	}
	file.write_all(b"\n").expect("Failed to write LF.");
    }
}

fn rename_index() {
    let src = index_file_path("words", ".old");
    if let Err(why) = fs::remove_file(src) {
	eprintln!("couldn't remove {}: {}", "index.words.txt.old", why);
    }
    let src = index_file_path("matrix", ".old");
    if let Err(why) = fs::remove_file(src) {
	eprintln!("couldn't remove {}: {}", "index.matrix.txt.old", why);
    }
    let src = index_file_path("words", "");
    let dst = index_file_path("words", ".old");
    fs::rename(src, dst).expect("rename failed");
    let src = index_file_path("matrix", "");
    let dst = index_file_path("matrix", ".old");
    fs::rename(src, dst).expect("rename failed");
    let src = index_file_path("words", ".new");
    let dst = index_file_path("words", "");
    fs::rename(src, dst).expect("rename failed");
    let src = index_file_path("matrix", ".new");
    let dst = index_file_path("matrix", "");
    fs::rename(src, dst).expect("rename failed");
}

fn main() {
    let dict = get_dict();

    let mut index_words = read_index_words();
    let mut index_matrix = read_index_matrix();

    for inp in &env::args().collect::<Vec<String>>()[1..] {
	let inp_path = Path::new(&inp);
	let mut file = File::open(&inp_path).expect("Failed to open file.");

	let mut buf = String::new();
	if let Err(why) = file.read_to_string(&mut buf) {
	    panic!("couldn't read {}: {}", inp_path.display(), why);
	}
	let buf = replace_lf(&buf);
	
	let set = tokenize(buf, &dict);
	
	let mut word_ids = HashSet::<u32>::new();
	for word in set.iter() {
	    let word_id: u32 = match index_words.get(word) {
		Some(id) => *id,
		None => {
		    let new_id = index_words.len() as u32;
		    index_words.insert(word.clone(), new_id);
		    new_id
		}
	    };

	    word_ids.insert(word_id);
	}
	index_matrix.insert(inp.clone(), word_ids);
    }

    write_index_words(index_words);
    write_index_matrix(index_matrix);

    rename_index();
}
