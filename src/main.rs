use std::path::PathBuf;
use std::collections::HashSet;
use sudachi::prelude::MorphemeList;
use sudachi::config::Config;
use sudachi::analysis::Mode;
use sudachi::analysis::stateful_tokenizer::StatefulTokenizer;
use sudachi::dic::dictionary::JapaneseDictionary;

fn tokenize(string: String) -> HashSet<String> {
    let config = Config::new(
	Some(PathBuf::from("./t/sudachi.rs/resources/sudachi.json")),
	Some(PathBuf::from("./t/sudachi.rs/resources")),
	Some(PathBuf::from("./t/sudachi.rs/resources/system.dic")),
    ).expect("Failed to load config file");
    let dict = JapaneseDictionary::from_cfg(&config)
	.unwrap_or_else(|e| panic!("Failed to create dictionary: {:?}", e));

    let mut analyzer = StatefulTokenizer::new(&dict, Mode::C);
    analyzer.reset().push_str(&string[..]);
    analyzer.do_tokenize()
	.unwrap_or_else(|_| panic!("Failed to tokenize."));
    let mut morphs = MorphemeList::empty(analyzer.dict_clone());
    morphs.collect_results(&mut analyzer)
	.unwrap_or_else(|_| panic!("Failed to collect results."));
    let mut set = HashSet::<String>::new();
    for m in morphs.iter() {
	set.insert(m.surface().to_string());
	set.insert(m.normalized_form().to_string());
    }
    set
}

fn main() {
    let ss = String::from("今日は東京駅に行きます。");
    let set = tokenize(ss);
    for s in set.iter() {
	println!("{}", s);
    }
}
