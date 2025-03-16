use kuchiki::traits::TendrilSink;
use kuchiki::*;
use lol_html::element;
use lol_html::{html_content::ContentType, HtmlRewriter, Settings};
use select::document::Document;
use select::predicate::Class;
use select::predicate::Name;
use select::predicate::Predicate;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Read, Write};
use uuid::Uuid;

fn main() {
    // 閱讀file
    let mut file = File::open("out.html").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    findUseLaw(&contents);

    let mut map: LawHash = LawHash {
        inner: HashMap::new(),
    };
    let law1 = usinglaw::new("民法".to_string(), "1".to_string());
    let law2 = usinglaw::new("刑法".to_string(), "2".to_string());
    let law3 = usinglaw::new("民法".to_string(), "1".to_string());
    let law4 = usinglaw::new("民法".to_string(), "4".to_string());
    map.insert("民法".to_string(), "1".to_string());
    map.insert("民法".to_string(), "2".to_string());
    println!("{}", map.format());

    /*
    //2.set uuid
    let mut output = Vec::new();
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!("h2, h3", |el| {
                if !el.has_attribute("id") {
                    let id = Uuid::new_v4().to_string();
                    el.set_attribute("id", &id);
                }
                Ok(())
            })],
            ..Settings::default()
        },
        |chunk: &[u8]| output.extend_from_slice(chunk),
    );

    rewriter.write(contents.as_bytes()).unwrap();
    rewriter.end().unwrap();
    contents = String::from_utf8(output).unwrap();

    //找出所有h2、h3
    let mut heading_vec = Vec::new();
    let document = Document::from(contents.as_str());
    heading_vec.extend(document.find(Name("h2").or(Name("h3"))).map(|x| {
        let id = x.attr("id").unwrap_or("no");
        format!("<li><a href='#{}'>{}</a></li>", id, x.text())
    }));

    println!("{}", heading_vec.join("/"));

    let mut output = Vec::new();
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!("nav", |el| {
                el.set_inner_content(heading_vec.join("").as_str(), ContentType::Html);
                Ok(())
            })],
            ..Settings::default()
        },
        |chunk: &[u8]| output.extend_from_slice(chunk),
    );

    rewriter.write(contents.as_bytes()).unwrap();
    rewriter.end().unwrap();

    // 寫出
    let mut file = File::create("out.html").unwrap();
    file.write_all(&output).unwrap();
    */
}

fn findUseLaw(file_content: &str) {
    let document = Document::from(file_content);
    let mut uselaw_vec = Vec::new();
    document
        .find(Class("law-block-chapter"))
        .for_each(|node| uselaw_vec.push(node.text()));
    for law in uselaw_vec {
        println!("{law}");
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct usinglaw {
    chapter: String,
    num: String,
}

impl usinglaw {
    pub fn new(chapter: String, num: String) -> Self {
        usinglaw { chapter, num }
    }
}

struct LawHash {
    inner: HashMap<String, HashSet<usinglaw>>,
}

impl LawHash {
    pub fn format(self) -> String {
        let mut buffer = String::new();
        for (key, set) in self.inner {
            let ul = format!("<ul>{}", key);
            buffer.push_str(&ul);
            set.iter().for_each(|law| {
                let li = format!("<li>{}</li>", law.num.clone());
                buffer.push_str(&li);
            });
            buffer.push_str("</ul>")
        }
        buffer
    }

    pub fn insert(&mut self, chapter: String, num: String) {
        self.inner
            .entry(chapter.clone())
            .or_default()
            .insert(usinglaw::new(chapter, num));
    }
}
