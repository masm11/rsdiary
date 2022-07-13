use std::path::PathBuf;
use std::collections::HashSet;
use std::collections::HashMap;
use sudachi::prelude::MorphemeList;
use sudachi::config::Config;
use sudachi::analysis::Mode;
use sudachi::analysis::stateful_tokenizer::StatefulTokenizer;
use sudachi::dic::dictionary::JapaneseDictionary;

/*
ors    = ands ( `OR` ors )*
ands   = nots ( `AND` ands )*
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

pub struct Parser<'a> {
    analyzer: &'a StatefulTokenizer<&'a JapaneseDictionary>,
    words: &'a HashMap<String, u32>,
    matrix: &'a HashMap<String, HashSet<u32>>,
}

impl<'a> Parser<'a> {
    pub fn new(analyzer: &'a StatefulTokenizer<&'a JapaneseDictionary>,
	       words: &'a HashMap<String, u32>,
	       matrix: &'a HashMap<String, HashSet<u32>>) -> Parser<'a> {
	Parser {
	    analyzer,
	    words,
	    matrix,
	}
    }

    fn get_token<'b>(&self, tokens: &Vec<&'b str>, pos: usize) -> TokenType<'b> {
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
    
    pub fn parse(&self, string: String) -> HashSet<String> {
	let tokens: Vec<&str> = string.split_ascii_whitespace().collect();
	let mut pos: usize = 0;
	match self.ors(&tokens, &mut pos) {
	    Some(r) => {
		if pos != tokens.len() {
		    panic!("syntax error!");
		}
		r
	    },
	    None => panic!("syntax error!"),
	}
    }
    
    fn ors(&self, tokens: &Vec<&str>, r_pos: &mut usize) -> Option<HashSet<String>> {
	let mut pos = *r_pos;
	let ands = self.ands(tokens, &mut pos);
	let mut ands = match ands {
	    Some(ands) => ands,
	    None => return None,
	};
	loop {
	    match self.get_token(tokens, pos) {
		TokenType::Or => {
		    let pos_at_or = pos;
		    pos += 1;
		    match self.ors(tokens, &mut pos) {
			Some(ors) => {
			    // union
			    for o in ors {
				ands.insert(o);
			    }
			},
			None => {
			    *r_pos = pos_at_or;
			    return Some(ands);
			},
		    };
		},
		_ => {
		    *r_pos = pos;
		    return Some(ands);
		}
	    }
	}
    }
    
    fn ands(&self, tokens: &Vec<&str>, r_pos: &mut usize) -> Option<HashSet<String>> {
	let mut pos = *r_pos;
	let nots = self.nots(tokens, &mut pos);
	let mut nots = match nots {
	    Some(nots) => nots,
	    None => return None,
	};
	loop {
	    match self.get_token(tokens, pos) {
		TokenType::And => {
		    let pos_at_and = pos;
		    pos += 1;
		    match self.ands(tokens, &mut pos) {
			Some(ands) => {
			    nots = HashSet::from_iter(nots.intersection(&ands).cloned());
			},
			None => {
			    *r_pos = pos_at_and;
			    return Some(nots);
			},
		    };
		},
		_ => {
		    *r_pos = pos;
		    return Some(nots);
		}
	    }
	}
    }
    
    fn nots(&self, tokens: &Vec<&str>, r_pos: &mut usize) -> Option<HashSet<String>> {
	let mut pos = *r_pos;

	match self.get_token(tokens, pos) {
	    TokenType::Not => {
		pos += 1;
		let mut nots = self.nots(tokens, &mut pos);
		match nots {
		    Some(some_nots) => {
			let all = HashSet::from_iter(self.matrix.keys().cloned());
			let res = HashSet::from_iter(all.difference(&some_nots).cloned());
			*r_pos = pos;
			return Some(res);
		    },
		    None => return None,
		}
	    },
	    _ => {
		let parens = self.parens(tokens, &mut pos);
		match parens {
		    Some(_) => {
			*r_pos = pos;
			return parens;
		    },
		    _ => {
			return None;
		    }
		}
	    },
	}
    }

    fn parens(&self, tokens: &Vec<&str>, r_pos: &mut usize) -> Option<HashSet<String>> {
	let mut pos = *r_pos;
	
	match self.get_token(tokens, pos) {
	    TokenType::Lpar => {
		pos += 1;
		let ors = self.ors(tokens, &mut pos);
		match ors {
		    Some(_) => {
			match self.get_token(tokens, pos) {
			    TokenType::Rpar => {
				pos += 1;
				*r_pos = pos;
				return ors;
			    },
			    _ => {
				return None;
			    },
			}
		    },
		    None => {
			return None;
		    },
		}
	    },
	    _ => {
		let word = self.word(tokens, &mut pos);
		match word {
		    Some(_) => {
			*r_pos = pos;
			return word;
		    },
		    _ => {
			return None;
		    }
		}
	    },
	}
    }

    fn word(&self, tokens: &Vec<&str>, r_pos: &mut usize) -> Option<HashSet<String>> {
	let mut pos = *r_pos;
	match self.get_token(tokens, pos) {
	    TokenType::Other(_tkn) => {
		return Some(HashSet::<String>::new());		// FIXME: tokenize
	    },
	    _ => {
		return None;
	    },
	}
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
	    Some(PathBuf::from("./t/sudachi.rs/resources/sudachi.json")),
	    Some(PathBuf::from("./t/sudachi.rs/resources")),
	    Some(PathBuf::from("./t/sudachi.rs/resources/system.dic")),
	).expect("Failed to load config file");
	JapaneseDictionary::from_cfg(&config).expect("Failed to read dict.")
    }

    #[test]
    fn simple() {
	let dict = get_dict();
	let mut analyzer = StatefulTokenizer::new(&dict, Mode::C);

	let mut words = HashMap::<String, u32>::new();
	words.insert(String::from("今日"), 1);
	words.insert(String::from("は"), 2);
	let mut mat = HashMap::<String, HashSet<u32>>::new();
	mat.insert(String::from("kyoha.txt"), set!{1, 2});
	mat.insert(String::from("ha.txt"), set!{1});
	let mut parser = Parser::new(&analyzer, &words, &mat);
	let result = parser.parse(String::from("今日は"));
	assert_eq!(result, set!{String::from("kyoha.txt")});
    }
}
