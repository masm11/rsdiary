use std::env;
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

fn replace_lf(buf: &String) -> String {
    buf.replace("\n", " ")
	.replace("\r", " ")
	.replace("\t", " ")
}

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
    let mut index = match File::create(&index_path) {
	Err(why) => panic!("couldn't create {}: {}", index_path.display(), why),
	Ok(index) => index,
    };

    for inp in &env::args().collect::<Vec<String>>()[1..] {
	let inp_path = Path::new(&inp);
	let mut file = match File::open(&inp_path) {
	    Err(why) => panic!("couldn't open {}: {}", inp, why),
	    Ok(file) => file,
	};
	let mut buf = String::new();
	if let Err(why) = file.read_to_string(&mut buf) {
	    panic!("couldn't read {}: {}", inp_path.display(), why);
	}
	
	let buf = replace_lf(&buf);

	if let Err(why) = index.write_all(inp.as_bytes()) {
	    panic!("couldn't write to {}: {}", index_path.display(), why);
	}
	if let Err(why) = index.write_all("\t".as_bytes()) {
	    panic!("couldn't write to {}: {}", index_path.display(), why);
	}
	
	let set = tokenize(buf, &dict);
	
	for s in set.iter() {
	    if let Err(why) = index.write_all(s.as_bytes()) {
		panic!("couldn't write to {}: {}", index_path.display(), why);
	    }
	    if let Err(why) = index.write_all(b" ") {
		panic!("couldn't write to {}: {}", index_path.display(), why);
	    }
	}
    }

}
