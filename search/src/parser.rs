use std::path::PathBuf;
use std::collections::HashSet;
use std::collections::HashMap;
use sudachi::prelude::MorphemeList;
use sudachi::config::Config;
use sudachi::analysis::Mode;
use sudachi::analysis::stateful_tokenizer::StatefulTokenizer;
use sudachi::dic::dictionary::JapaneseDictionary;

/*
ors    = ands ( `OR` ands )*
ands   = nots ( `AND` nots )*
       | nots ( nots )*           ここの文法、どうしたらいいのかなぁ
nots   = `NOT` nots
       | parens
parens = `(` ors `)`
       | word
word   = WORD

( あいう AND たちつ ) ( かきく OR さしす )
NOT ( あいう AND たちつ ) ( かきく OR さしす )
NOT ( あいう AND たちつ ) AND ( かきく OR さしす )
( ( あいう OR たちつ ) ( かきく OR さしす ) ) ( なにぬ AND はひふ )
*/

enum TokenType<'a> {
    None,
    And,
    Or,
    Not,
    Lpar,
    Rpar,
    Other(&'a str),
}

enum RetVal {
    Tree(HashSet<String>, usize),
    None,
}

pub struct Parser<'a, 'b> {
    analyzer: &'a mut StatefulTokenizer<&'b JapaneseDictionary>,
    words: &'a HashMap<String, u32>,
    matrix: &'a HashMap<String, HashSet<u32>>,
    imat: HashMap<u32, HashSet<String>>,
}

impl<'a, 'b> Parser<'a, 'b> {
    pub fn new(analyzer: &'a mut StatefulTokenizer<&'b JapaneseDictionary>,
	       words: &'a HashMap<String, u32>,
	       matrix: &'a HashMap<String, HashSet<u32>>) -> Parser<'a, 'b> {
	let mut imat = HashMap::<u32, HashSet<String>>::new();
	for (fname, word_ids) in matrix {
	    for word_id in word_ids {
		let set = match imat.get_mut(word_id) {
		    Some(set) => set,
		    None => {
			imat.insert(*word_id, HashSet::<String>::new());
			match imat.get_mut(word_id) {
			    Some(set) => set,
			    None => panic!("why?"),
			}
		    }
		};
		set.insert(fname.clone());
	    }
	}

	Parser {
	    analyzer,
	    words,
	    matrix,
	    imat,
	}
    }

    fn get_token<'z>(&self, tokens: &Vec<&'z str>, pos: usize) -> TokenType<'z> {
	if pos >= tokens.len() {
	    return TokenType::None;
	}
	let s = tokens[pos];
	if s == "AND" {
	    return TokenType::And;
	}
	if s == "OR" {
	    return TokenType::Or;
	}
	if s == "NOT" {
	    return TokenType::Not;
	}
	if s == "(" {
	    return TokenType::Lpar;
	}
	if s == ")" {
	    return TokenType::Rpar;
	}
	return TokenType::Other(s);
    }
    
    pub fn parse(&mut self, string: String) -> HashSet<String> {
	let tokens: Vec<&str> = string.split_ascii_whitespace().collect();
	match self.ors(&tokens, 0) {
	    RetVal::Tree(r, rpos) => {
		if rpos != tokens.len() {
		    panic!("syntax error! (length not match, {}, {})", rpos, tokens.len());
		}
		r
	    },
	    RetVal::None => panic!("syntax error! (parse error)"),
	}
    }
    
    fn ors(&mut self, tokens: &Vec<&str>, pos: usize) -> RetVal {
	let ands = self.ands(tokens, pos);
	let (mut ands, mut pos) = match ands {
	    RetVal::Tree(ands, pos) => (ands, pos),
	    RetVal::None => return RetVal::None,
	};
	loop {
	    match self.get_token(tokens, pos) {
		TokenType::Or => {
		    let pos_at_or = pos;
		    pos += 1;
		    match self.ands(tokens, pos) {
			RetVal::Tree(rands, rpos) => {
			    ands = HashSet::from_iter(ands.union(&rands).cloned());
			    pos = rpos;
			},
			RetVal::None => {
			    return RetVal::Tree(ands, pos_at_or);
			},
		    };
		},
		_ => {
		    return RetVal::Tree(ands, pos);
		},
	    }
	}
    }
    
    fn ands(&mut self, tokens: &Vec<&str>, pos: usize) -> RetVal {
	let nots = self.nots(tokens, pos);
	let (mut nots, mut pos) = match nots {
	    RetVal::Tree(nots, pos) => (nots, pos),
	    RetVal::None => return RetVal::None,
	};
	loop {
	    match self.get_token(tokens, pos) {
		TokenType::And => {
		    let pos_at_and = pos;
		    pos += 1;
		    match self.nots(tokens, pos) {
			RetVal::Tree(rnots, rpos) => {
			    nots = HashSet::from_iter(nots.intersection(&rnots).cloned());
			    pos = rpos;
			},
			RetVal::None => {
			    return RetVal::Tree(nots, pos_at_and);
			},
		    };
		},
		TokenType::Or => {
		    return RetVal::Tree(nots, pos);
		},
		TokenType::Rpar => {
		    return RetVal::Tree(nots, pos);
		},
		TokenType::None => {
		    return RetVal::Tree(nots, pos);
		},
		_ => {
		    match self.nots(tokens, pos) {
			RetVal::Tree(rnots, rpos) => {
			    nots = HashSet::from_iter(nots.intersection(&rnots).cloned());
			    pos = rpos;
			},
			RetVal::None => {
			    return RetVal::Tree(nots, pos);
			},
		    };
		}
	    }
	}
    }
    
    fn nots(&mut self, tokens: &Vec<&str>, mut pos: usize) -> RetVal {
	match self.get_token(tokens, pos) {
	    TokenType::Not => {
		pos += 1;
		let nots = self.nots(tokens, pos);
		match nots {
		    RetVal::Tree(some_nots, rpos) => {
			let all = self.all();
			let res = HashSet::from_iter(all.difference(&some_nots).cloned());
			return RetVal::Tree(res, rpos);
		    },
		    RetVal::None => return RetVal::None,
		}
	    },
	    _ => {
		match self.parens(tokens, pos) {
		    RetVal::Tree(parens, pos) => {
			return RetVal::Tree(parens, pos);
		    },
		    _ => {
			return RetVal::None;
		    }
		}
	    },
	}
    }

    fn parens(&mut self, tokens: &Vec<&str>, mut pos: usize) -> RetVal {
	match self.get_token(tokens, pos) {
	    TokenType::Lpar => {
		pos += 1;
		let ors = self.ors(tokens, pos);
		match ors {
		    RetVal::Tree(ors, mut pos) => {
			match self.get_token(tokens, pos) {
			    TokenType::Rpar => {
				pos += 1;
				return RetVal::Tree(ors, pos);
			    },
			    _ => {
				return RetVal::None;
			    },
			}
		    },
		    RetVal::None => {
			return RetVal::None;
		    },
		}
	    },
	    _ => {
		let word = self.word(tokens, pos);
		match word {
		    RetVal::Tree(_, _) => {
			return word;
		    },
		    _ => {
			return RetVal::None;
		    }
		}
	    },
	}
    }

    fn word(&mut self, tokens: &Vec<&str>, mut pos: usize) -> RetVal {
	match self.get_token(tokens, pos) {
	    TokenType::Other(tkn) => {
		self.analyzer
		    .reset()
		    .push_str(tkn);
		self.analyzer
		    .do_tokenize()
		    .expect("Failed to tokenize.");
		let mut morphs = MorphemeList::empty(self.analyzer.dict_clone());
		morphs.collect_results(self.analyzer)
		    .expect("Failed to collect results.");
		let mut retval = self.all();
		for m in morphs.iter() {
		    let s = m.surface().to_string();
		    let empty = HashSet::<String>::new();
		    let fns = match self.words.get(&s) {
			Some(word_id) => {
			    match self.imat.get(word_id) {
				Some(fns) => fns,
				None => &empty,	// 単語は知ってるけど、該当文書がない
			    }
			},
			None => &empty,	// 未知語
		    };
		    retval = HashSet::from_iter(retval.intersection(fns).cloned());
		}
		pos += 1;
		return RetVal::Tree(retval, pos);
	    },
	    _ => {
		return RetVal::None;
	    },
	}
    }

    fn all(&self) -> HashSet<String> {
	HashSet::from_iter(self.matrix.keys().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! set {
	($( $x: expr ), *) => {{
	    let mut _set = ::std::collections::HashSet::new();
	    $( _set.insert($x); )*
	    _set
	}}
    }

    fn get_dict() -> JapaneseDictionary {
	let config = Config::new(
	    Some(PathBuf::from("../t/sudachi.rs/resources/sudachi.json")),
	    Some(PathBuf::from("../t/sudachi.rs/resources")),
	    Some(PathBuf::from("../t/sudachi.rs/resources/system.dic")),
	).expect("Failed to load config file");
	JapaneseDictionary::from_cfg(&config).expect("Failed to read dict.")
    }

    #[test]
    fn test<'b>() {
	let dict = get_dict();
	let mut analyzer = StatefulTokenizer::new(&dict, Mode::C);
	simple(&mut analyzer);
	and(&mut analyzer);
	or(&mut analyzer);
	complex1(&mut analyzer);
	complex2(&mut analyzer);
	complex3(&mut analyzer);
	complex4(&mut analyzer);
    }
    
    fn simple<'a, 'b>(analyzer: &'a mut StatefulTokenizer<&'b JapaneseDictionary>) {
	let mut words = HashMap::<String, u32>::new();
	words.insert(String::from("今日"), 1);
	words.insert(String::from("は"), 2);
	let mut mat = HashMap::<String, HashSet<u32>>::new();
	mat.insert(String::from("kyoha.txt"), set!{1, 2});
	mat.insert(String::from("ha.txt"), set!{1});
	let mut parser = Parser::new(analyzer, &words, &mat);
	let result = parser.parse(String::from("今日は"));
	
	assert_eq!(result, set!{String::from("kyoha.txt")});
    }

    fn and<'a, 'b>(analyzer: &'a mut StatefulTokenizer<&'b JapaneseDictionary>) {
	let mut words = HashMap::<String, u32>::new();
	words.insert(String::from("今日"), 1);
	words.insert(String::from("は"), 2);
	words.insert(String::from("良い"), 3);
	words.insert(String::from("天気"), 4);
	words.insert(String::from("でし"), 5);
	words.insert(String::from("です"), 6);
	words.insert(String::from("た"), 7);
	words.insert(String::from("悪い"), 8);
	let mut mat = HashMap::<String, HashSet<u32>>::new();
	mat.insert(String::from("bad.txt"), set!{1, 2, 8, 4, 5, 6, 7});
	mat.insert(String::from("good.txt"), set!{1, 2, 3, 4, 5, 6, 7});
	let mut parser = Parser::new(analyzer, &words, &mat);
	let result = parser.parse(String::from("今日 AND 良い AND 天気"));

	assert_eq!(result, set!{String::from("good.txt")});
    }

    fn or<'a, 'b>(analyzer: &'a mut StatefulTokenizer<&'b JapaneseDictionary>) {
	let mut words = HashMap::<String, u32>::new();
	words.insert(String::from("今日"), 1);
	words.insert(String::from("は"), 2);
	words.insert(String::from("良い"), 3);
	words.insert(String::from("天気"), 4);
	words.insert(String::from("でし"), 5);
	words.insert(String::from("です"), 6);
	words.insert(String::from("た"), 7);
	words.insert(String::from("悪い"), 8);
	let mut mat = HashMap::<String, HashSet<u32>>::new();
	mat.insert(String::from("bad.txt"), set!{1, 2, 8, 4, 5, 6, 7});
	mat.insert(String::from("good.txt"), set!{1, 2, 3, 4, 5, 6, 7});
	let mut parser = Parser::new(analyzer, &words, &mat);
	let result = parser.parse(String::from("今日 AND ( 良い OR 悪い ) AND 天気"));

	assert_eq!(result, set!{String::from("good.txt"), String::from("bad.txt")});
    }

    fn get_complex_index(words: &mut HashMap<String, u32>, mat: &mut HashMap<String, HashSet<u32>>) {
	words.insert(String::from("優子"), 1);
	words.insert(String::from("愛子"), 2);
	words.insert(String::from("涼子"), 3);
	words.insert(String::from("恵子"), 4);
	words.insert(String::from("真知子"), 5);
	words.insert(String::from("和美"), 6);

	for i6 in 0..2 {
	    for i5 in 0..2 {
		for i4 in 0..2 {
		    for i3 in 0..2 {
			for i2 in 0..2 {
			    for i1 in 0..2 {
				let mut set = HashSet::<u32>::new();
				if i1 == 1 { set.insert(1); }	// 優子 (1)
				if i2 == 1 { set.insert(2); }	// 愛子 (2)
				if i3 == 1 { set.insert(3); }	// 涼子 (4)
				if i4 == 1 { set.insert(4); }	// 恵子 (8)
				if i5 == 1 { set.insert(5); }	// 真知子 (16)
				if i6 == 1 { set.insert(6); }	// 和美 (32)
				let no = 32 * i6 + 16 * i5 + 8 * i4 + 4 * i3 + 2 * i2 + 1 * i1;
				let fname = format!("file{}.txt", no);
				mat.insert(fname, set);
			    }
			}
		    }
		}
	    }
	}
    }

    fn complex1<'a, 'b>(analyzer: &'a mut StatefulTokenizer<&'b JapaneseDictionary>) {
	let mut words = HashMap::<String, u32>::new();
	let mut mat = HashMap::<String, HashSet<u32>>::new();
	get_complex_index(&mut words, &mut mat);
	let mut parser = Parser::new(analyzer, &words, &mat);
	let result = parser.parse(String::from("( 優子 AND 恵子 ) ( 愛子 OR 涼子 )"));

	let fids_vec = vec![11, 13, 15, 27, 29, 31, 43, 45, 47, 59, 61, 63];
	let fnames_iter = fids_vec.iter().map(|id| format!("file{}.txt", id));
	let fnames = HashSet::from_iter(fnames_iter);

	assert_eq!(result, fnames);
    }

    fn complex2<'a, 'b>(analyzer: &'a mut StatefulTokenizer<&'b JapaneseDictionary>) {
	let mut words = HashMap::<String, u32>::new();
	let mut mat = HashMap::<String, HashSet<u32>>::new();
	get_complex_index(&mut words, &mut mat);
	let mut parser = Parser::new(analyzer, &words, &mat);
	let result = parser.parse(String::from("NOT ( 優子 AND 恵子 ) ( 愛子 OR 涼子 )"));

	let fids_vec = vec![2, 3, 4, 5, 6, 7, 10, 12, 14, 18, 19, 20, 21, 22, 23, 26, 28, 30, 34, 35, 36, 37, 38, 39, 42, 44, 46, 50, 51, 52, 53, 54, 55, 58, 60, 62];

	let fnames_iter = fids_vec.iter().map(|id| format!("file{}.txt", id));
	let fnames = HashSet::from_iter(fnames_iter);

	assert_eq!(result, fnames);
    }

    fn complex3<'a, 'b>(analyzer: &'a mut StatefulTokenizer<&'b JapaneseDictionary>) {
	let mut words = HashMap::<String, u32>::new();
	let mut mat = HashMap::<String, HashSet<u32>>::new();
	get_complex_index(&mut words, &mut mat);
	let mut parser = Parser::new(analyzer, &words, &mat);
	let result = parser.parse(String::from("NOT ( 優子 AND 恵子 ) AND ( 愛子 OR 涼子 )"));

	let fids_vec = vec![2, 3, 4, 5, 6, 7, 10, 12, 14, 18, 19, 20, 21, 22, 23, 26, 28, 30, 34, 35, 36, 37, 38, 39, 42, 44, 46, 50, 51, 52, 53, 54, 55, 58, 60, 62];

	let fnames_iter = fids_vec.iter().map(|id| format!("file{}.txt", id));
	let fnames = HashSet::from_iter(fnames_iter);

	assert_eq!(result, fnames);
    }

    fn complex4<'a, 'b>(analyzer: &'a mut StatefulTokenizer<&'b JapaneseDictionary>) {
	let mut words = HashMap::<String, u32>::new();
	let mut mat = HashMap::<String, HashSet<u32>>::new();
	get_complex_index(&mut words, &mut mat);
	let mut parser = Parser::new(analyzer, &words, &mat);
	let result = parser.parse(String::from("( ( 優子 OR 恵子 ) ( 愛子 OR 涼子 ) ) ( 真知子 AND 和美 )"));

	let fids_vec = vec![51, 53, 55, 58, 59, 60, 61, 62, 63];

	let fnames_iter = fids_vec.iter().map(|id| format!("file{}.txt", id));
	let fnames = HashSet::from_iter(fnames_iter);

	assert_eq!(result, fnames);
    }
}
