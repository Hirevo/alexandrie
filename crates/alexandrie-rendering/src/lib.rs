use cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag};
use syntect::easy::HighlightLines;
use syntect::html::{
    start_highlighted_html_snippet, styled_line_to_highlighted_html, IncludeBackground,
};

pub mod config;

use crate::config::SyntectState;

/// Stores the location and level of an header tag inside a Markdown document.
#[derive(Debug, Clone, PartialEq)]
pub struct HeaderRef {
    /// The header tag's level (1 to 6).
    pub level: HeadingLevel,
    /// The header tag's start index in the event list.
    pub start: usize,
    /// The header tag's end index in the event list.
    pub end: usize,
}

/// Renders a Markdown document to HTML using the provided configuration.
pub fn render_readme(config: &SyntectState, contents: &str) -> String {
    let mut highlighter: Option<HighlightLines> = None;
    let events = Parser::new_ext(contents, Options::all());
    let mut events = events
        .map(|event| match event {
            Event::Text(text) => highlighter
                .as_mut()
                .and_then(|highlighter| highlighter.highlight_line(&text, &config.syntaxes).ok())
                .and_then(|highlighted| {
                    styled_line_to_highlighted_html(&highlighted, IncludeBackground::Yes).ok()
                })
                .map(|html| Event::Html(html.into()))
                .unwrap_or_else(|| Event::Text(text)),
            Event::Start(Tag::CodeBlock(info)) => {
                let theme = &config.themes.themes[&config.theme_name];

                highlighter = Some(match info {
                    CodeBlockKind::Fenced(lang) => {
                        let syntax = config
                            .syntaxes
                            .find_syntax_by_token(lang.as_ref())
                            .unwrap_or_else(|| config.syntaxes.find_syntax_plain_text());
                        HighlightLines::new(syntax, theme)
                    }
                    CodeBlockKind::Indented => {
                        HighlightLines::new(config.syntaxes.find_syntax_plain_text(), theme)
                    }
                });
                let snippet = start_highlighted_html_snippet(theme);
                Event::Html(snippet.0.into())
            }
            Event::End(Tag::CodeBlock(_)) => {
                highlighter = None;
                Event::Html("</pre>".into())
            }
            _ => event,
        })
        .collect::<Vec<_>>();

    let header_count = events.iter().fold(0usize, |acc, event| match event {
        Event::Start(Tag::Heading(_, _, _)) => acc + 1,
        _ => acc,
    });
    let mut header_refs = Vec::with_capacity(header_count);

    events
        .iter()
        .enumerate()
        .for_each(|(idx, event)| match event {
            Event::Start(Tag::Heading(level, _, _)) => {
                header_refs.push(HeaderRef {
                    level: *level,
                    start: idx,
                    end: 0,
                });
            }
            Event::End(Tag::Heading(_, _, _)) => {
                header_refs.last_mut().unwrap().end = idx;
            }
            _ => {}
        });

    for href in header_refs.into_iter() {
        fn get_text(events: &[Event]) -> String {
            events.iter().fold(String::new(), |acc, event| match event {
                Event::Text(text) | Event::Code(text) => acc + text,
                _ => acc,
            })
        }

        let mut id = get_text(&events[(href.start + 1)..href.end])
            .replace(' ', "-")
            .replace('"', "\"");
        id.make_ascii_lowercase();
        events[href.start] = Event::Html(
            format!(
                r##"<h{0} class="header" id="{1}"><a class="permalink" href="#{1}">#</a>&nbsp;"##,
                href.level, id
            )
            .into(),
        );
        events[href.end] = Event::Html(format!("</h{0}>", href.level).into());
    }

    let mut html = String::new();
    cmark::html::push_html(&mut html, events.into_iter());

    ammonia::clean(html.as_str())
}
