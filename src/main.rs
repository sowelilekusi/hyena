//use std::env;
use std::fs;
use std::io::Write;
use inkjet::Language;
use inkjet::Highlighter;
use inkjet::formatter;
use comrak::{Arena, parse_document, format_html, Options};
use comrak::nodes::{AstNode, NodeValue};
use gray_matter::Matter;
use gray_matter::engine::YAML;
//use serde::Deserialize;

fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
	where F : Fn(&'a AstNode<'a>) {
	f(node);
	for c in node.children() {
		iter_nodes(c, f);
	}
}

fn main() {
	let paths = fs::read_dir("./posts/").unwrap();

	for path in paths {
		let path_buf = path.expect("Failed to read file path!").path();
		let path_str = path_buf.to_str().expect("Failed to convert PathBuf to str!");
		process_blog_post(path_str);
	}
}

fn process_blog_post(md_file_path: &str) {	
	let markdown_source = fs::read_to_string(md_file_path)
		.expect("Failed to read from file!");

	let matter = Matter::<YAML>::new();
	let result = matter.parse(&markdown_source);

	let blog_date = result.data.as_ref().unwrap()["date"].as_string().expect("Failed to extract blog date");
	let blog_title = result.data.as_ref().unwrap()["title"].as_string().expect("Failed to extract blog title");

	println!("Generating HTML for: \"{blog_title}\"");

	let mut options = Options::default();
	options.render.unsafe_ = true;

	let arena = Arena::new();

	let root = parse_document(
		&arena,
		&result.content,
		&options
	);
	
	iter_nodes(root, &|node| {
		match &mut node.data.borrow_mut().value {
			&mut NodeValue::CodeBlock(ref mut code) => {
				let orig = std::mem::replace(&mut code.literal, String::new());
				let mut highlighter = Highlighter::new();
				let html = highlighter.highlight_to_string(
					Language::Gdscript,
					&formatter::Html,
					orig
				).expect("Couldn't parse code to HTML!");
				code.literal.push_str(&html);
			}
			_ => (),
		}
	});

	let mut converted_html = vec![];
	format_html(root, &options, &mut converted_html).unwrap();

	//Fixes the fact that the comrak fucks up the formatting that inkjet does
	let fixed_html = std::str::from_utf8(&converted_html).expect("Couldn't convert byte vector to string");
	let fixed_html = fixed_html.replace("&lt;", "<");
	let fixed_html = fixed_html.replace("&gt;", ">");
	let fixed_html = fixed_html.replace("&quot;", r#"""#);
	let fixed_html = fixed_html.replace("&amp;quot;", r#"""#);

	let template = fs::read_to_string("templates/weblog-post.html")
		.expect("Failed to read from file!");

	let final_html = template.replace("CONTENTS", &fixed_html);
	let final_html = final_html.replace("TITLE", &blog_title);

	let slug = blog_title.replace(" ", "-");
	let slug = slug.replace(":", "");
	let slug = slug.replace(",", "");
	let slug = slug.replace("?", "");
	let slug = slug.replace("!", "");
	let slug = str::to_lowercase(&slug);

	let output_file_name = format!("weblog-entries/{0}_{1}.html", &blog_date, &slug);

	let mut file = fs::File::create(&output_file_name)
		.expect("Failed to create output file!");

	file.write_all(final_html.as_bytes())
		.expect("Failed to write to file!");
}
