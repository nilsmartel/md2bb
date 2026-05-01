use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, HeadingLevel, CodeBlockKind};
use std::env;
use std::fs;
use std::io::{self, Read};

fn heading_size(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 6,
        HeadingLevel::H2 => 5,
        HeadingLevel::H3 => 4,
        HeadingLevel::H4 => 3,
        HeadingLevel::H5 => 2,
        HeadingLevel::H6 => 1,
    }
}

fn md_to_bbcode(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(input, options);
    let mut output = String::new();

    // Stack to track open tags that need closing
    let mut tag_stack: Vec<String> = Vec::new();
    // For list tracking: true = ordered, false = unordered
    let mut list_stack: Vec<Option<u64>> = Vec::new();

    for event in parser {
        match event {
            // ── Opening tags ──────────────────────────────────────────────
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}
                Tag::Heading { level, .. } => {
                    let size = heading_size(level);
                    output.push_str(&format!("[size={}][b]", size));
                    tag_stack.push(format!("[/b][/size]"));
                }
                Tag::Strong => {
                    output.push_str("[b]");
                    tag_stack.push("[/b]".to_string());
                }
                Tag::Emphasis => {
                    output.push_str("[i]");
                    tag_stack.push("[/i]".to_string());
                }
                Tag::Strikethrough => {
                    output.push_str("[s]");
                    tag_stack.push("[/s]".to_string());
                }
                Tag::Link { dest_url, title, .. } => {
                    let open = if title.is_empty() {
                        format!("[url={}]", dest_url)
                    } else {
                        format!("[url={}]", dest_url)
                    };
                    output.push_str(&open);
                    tag_stack.push("[/url]".to_string());
                }
                Tag::Image { dest_url, .. } => {
                    output.push_str(&format!("[img]{}[/img]", dest_url));
                    // Image alt text events will be swallowed; push empty close
                    tag_stack.push("__img__".to_string());
                }
                Tag::CodeBlock(kind) => {
                    match kind {
                        CodeBlockKind::Fenced(lang) if !lang.is_empty() => {
                            output.push_str(&format!("[code={}]", lang));
                            tag_stack.push("[/code]".to_string());
                        }
                        _ => {
                            output.push_str("[code]");
                            tag_stack.push("[/code]".to_string());
                        }
                    }
                }
                Tag::BlockQuote(_) => {
                    output.push_str("[quote]");
                    tag_stack.push("[/quote]".to_string());
                }
                Tag::List(start) => {
                    list_stack.push(start);
                    match start {
                        Some(_) => output.push_str("[list=1]\n"),
                        None    => output.push_str("[list]\n"),
                    }
                }
                Tag::Item => {
                    output.push_str("[*]");
                }
                Tag::Table(_) => {
                    output.push_str("[table]\n");
                    tag_stack.push("[/table]".to_string());
                }
                Tag::TableHead => {
                    output.push_str("[tr]\n");
                    tag_stack.push("[/tr]\n".to_string());
                }
                Tag::TableRow => {
                    output.push_str("[tr]\n");
                    tag_stack.push("[/tr]\n".to_string());
                }
                Tag::TableCell => {
                    output.push_str("[td]");
                    tag_stack.push("[/td]\n".to_string());
                }
                Tag::HtmlBlock => {}
                _ => {}
            },

            // ── Closing tags ──────────────────────────────────────────────
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    output.push_str("\n\n");
                }
                TagEnd::Heading(_) => {
                    if let Some(close) = tag_stack.pop() {
                        output.push_str(&close);
                    }
                    output.push_str("\n\n");
                }
                TagEnd::Strong
                | TagEnd::Emphasis
                | TagEnd::Strikethrough
                | TagEnd::Link
                | TagEnd::CodeBlock
                | TagEnd::BlockQuote(_)
                | TagEnd::Table
                | TagEnd::TableHead
                | TagEnd::TableRow
                | TagEnd::TableCell => {
                    if let Some(close) = tag_stack.pop() {
                        if close != "__img__" {
                            output.push_str(&close);
                        }
                    }
                }
                TagEnd::Image => {
                    // Pop the sentinel we pushed; actual content already written
                    tag_stack.pop();
                }
                TagEnd::List(_) => {
                    list_stack.pop();
                    output.push_str("[/list]\n");
                }
                TagEnd::Item => {
                    output.push('\n');
                }
                _ => {}
            },

            // ── Leaf events ───────────────────────────────────────────────
            Event::Text(text) => {
                // Don't emit alt text that is inside an [img] tag
                let inside_img = tag_stack.last().map(|s| s == "__img__").unwrap_or(false);
                if !inside_img {
                    output.push_str(&text);
                }
            }
            Event::Code(text) => {
                output.push_str(&format!("[icode]{}[/icode]", text));
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                // Pass raw HTML through unchanged (most forums strip it anyway)
                output.push_str(&html);
            }
            Event::SoftBreak => {
                output.push('\n');
            }
            Event::HardBreak => {
                output.push_str("\n\n");
            }
            Event::Rule => {
                output.push_str("[hr]\n");
            }
            _ => {}
        }
    }

    output
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let input = if args.len() >= 2 {
        // Read from file path given as first argument
        match fs::read_to_string(&args[1]) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file '{}': {}", args[1], e);
                std::process::exit(1);
            }
        }
    } else {
        // Read from stdin
        let mut buf = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buf) {
            eprintln!("Error reading stdin: {}", e);
            std::process::exit(1);
        }
        buf
    };

    print!("{}", md_to_bbcode(&input));
}
