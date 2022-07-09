use std::path::PathBuf;
use std::collections::HashSet;
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

    let ss = String::from("今日は東京駅に行きます。よろしくお願いします。");
    let set = tokenize(ss, &dict);
    for s in set.iter() {
	println!("{}", s);
    }
}
