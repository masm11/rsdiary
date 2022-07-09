use std::path::PathBuf;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;   // write_all
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
    JapaneseDictionary::from_cfg(&config)
	.unwrap_or_else(|e| panic!("Failed to create dictionary: {:?}", e))
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
	ana.do_tokenize()
	    .unwrap_or_else(|_| panic!("Failed to tokenize."));
	let mut morphs = MorphemeList::empty(ana.dict_clone());
	morphs.collect_results(ana)
	    .unwrap_or_else(|_| panic!("Failed to collect results."));
	for m in morphs.iter() {
	    set.insert(m.surface().to_string());
	    set.insert(m.normalized_form().to_string());
	}
    }
    set
}

fn main() {
    let dict = get_dict();

    let index_path = Path::new("index.txt");
    let mut file = match File::create(&index_path) {
	Err(why) => panic!("couldn't create {}: {}", index_path.display(), why),
	Ok(file) => file,
    };

    let ss = String::from("今日は東京駅に行きます。よろしくお願いします。");

    let set = tokenize(ss, &dict);
    for s in set.iter() {
	if let Err(why) = file.write_all(s.as_bytes()) {
            panic!("couldn't write to {}: {}", index_path.display(), why);
	}
	if let Err(why) = file.write_all(b" ") {
            panic!("couldn't write to {}: {}", index_path.display(), why);
	}
    }
}
