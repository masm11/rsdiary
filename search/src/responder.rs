use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::collections::HashSet;
use std::path::Path;
use serde::Serialize;
use tera::{Context, Tera};

#[derive(Serialize)]
struct ResultFile {
    path: String,
    url: String,
    title: String,
    summary: String,
}

impl ResultFile {
    fn new(path: String) -> Self {
	let url = Self::make_url(&path);
	let title = Self::make_title(&path);
	let summary = Self::make_summary(&path);
	Self {
	    path,
	    url,
	    title,
	    summary,
	}
    }
    fn make_url(path: &String) -> String {
	let p = Path::new(path);
	let name = p.file_stem().unwrap();
	let name = name.to_str().unwrap();
	format!("http://localhost/{}", name)
    }
    fn make_title(path: &String) -> String {
	let path = Path::new(path);
	let file = File::open(&path).expect("Failed to open split file.");
	let mut reader = BufReader::new(file);
	let mut line = String::new();
	reader.read_line(&mut line);
	line.trim().to_string()
    }
    fn make_summary(path: &String) -> String {
	let path = Path::new(path);
	let file = File::open(&path).expect("Failed to open split file.");
	let mut reader = BufReader::new(file);
	let mut buf = String::new();
	reader.read_line(&mut buf);
	buf.clear();
	for line in reader.lines() {
	    let line = line.unwrap();
	    let l = line.trim();
	    buf.push_str(l);
	    buf.push_str(" ");
	}
	buf
    }
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
	    let rf = ResultFile::new(f);
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
	    String::from("/home/masm/esdiary/split/202208/20220805p01.est"),
	    String::from("/home/masm/esdiary/split/202208/20220803p01.est")
	};
	let html = res.make_html(String::from("foo>bar"), 1, files);
	out(&html);
    }

    fn out(s: &String) {
	let mut file = File::create("/dev/tty").expect("create failed.");
	file.write_all(s.as_bytes()).expect("write_all failed.");
	file.flush().expect("flush failed.");
    }
}
