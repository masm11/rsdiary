use std::collections::HashSet;
use serde::Serialize;
use tera::{Context, Tera};

#[derive(Serialize)]
struct ResultFile {
    path: String,
/*
    url: String,
    title: String,
    summary: String,
*/
}

struct Responder {
}

impl Responder {
    fn new() -> Self {
	Responder {}
    }
    fn make_html(&self, q: String, page_no: i32, files: HashSet<String>) -> String {
	let mut tera = match Tera::new("templates/*.html") {
	    Ok(t) => t,
	    Err(e) => return format!("{:?}", e)
	};
	let mut list = Vec::<ResultFile>::new();
	for f in files {
	    let rf = ResultFile {
		path: f,
	    };
	    list.push(rf);
	}
	let mut ctxt = Context::new();
	ctxt.insert("q", &q);
	ctxt.insert("list", &list);
	let html = match tera.render("index.html", &ctxt) {
	    Ok(html) => html,
	    Err(e) => return format!("{:?}", e),
	};
	html
    }
    fn make_internal_error(&self) -> String {
	String::from("internal error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    macro_rules! set {
	($( $x: expr ), *) => {{
	    let mut _set = ::std::collections::HashSet::new();
	    $( _set.insert($x); )*
	    _set
	}}
    }

    #[test]
    fn test() {
	let res = Responder::new();
	let files = set!{
	    String::from("test1"),
	    String::from("test2"),
	    String::from("test3")
	};
	let html = res.make_html(String::from("foo\"bar"), 1, files);
	out(&html);
    }

    fn out(s: &String) {
	let mut file = File::create("/dev/tty").expect("create failed.");
	file.write_all(s.as_bytes()).expect("write_all failed.");
	file.flush().expect("flush failed.");
    }
}
