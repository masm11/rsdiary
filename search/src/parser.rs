use std::collections::HashSet;

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

pub struct Parser {
}

impl Parser {
    pub fn new() -> Parser {
	Parser {}
    }

    fn get_token<'a>(&self, tokens: &Vec<&'a str>, pos: usize) -> TokenType<'a> {
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
		let nots = self.nots(tokens, &mut pos);
		match nots {
		    Some(nots) => {
			// nots = nots.intersection(&i_nots);	// FIXME: all - nots
			*r_pos = pos;
			return Some(nots);
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
