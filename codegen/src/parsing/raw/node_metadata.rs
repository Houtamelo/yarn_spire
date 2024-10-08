use anyhow::{Result, anyhow};
use genco::prelude::{FormatInto, Rust};
use genco::{quote_in, Tokens};
use crate::parsing::macros::strip_start_then_trim;
use crate::UnparsedLine;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TrackingSetting {
	Always,
	Never,
}

impl FormatInto<Rust> for TrackingSetting {
	fn format_into(self, tokens: &mut Tokens<Rust>) {
		quote_in!(*tokens => $(match self {
			TrackingSetting::Always => "TrackingSetting::Always",
			TrackingSetting::Never => "TrackingSetting::Never",
		}))
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeMetadata {
	pub title: String,
	pub tags: Vec<String>,
	pub tracking: Option<TrackingSetting>,
	pub customs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MetaLine {
	Title(String),
	Tags(Vec<String>),
	Tracking(TrackingSetting),
	Custom(String),
}

fn parse_meta_line(source_line: &UnparsedLine) -> Result<MetaLine> {
	let mut text = source_line.text.trim();

	if strip_start_then_trim!(text, "title" | "Title" | "TITLE")
		&& strip_start_then_trim!(text, ':') {
		
		if !text.is_empty() {
			Ok(MetaLine::Title(text.to_string()))
		} else {
			Err(anyhow!(
				"Missing title name in declaration.\n\
				 A `title:` declaration was found but no title was provided.\n\
				 At line nº{}, `{}`\n\n\
				 Help: Provide a title.", source_line.line_number, source_line.text))
		}
	} else if strip_start_then_trim!(text, "tags" | "Tags" | "TAGS")
		&& strip_start_then_trim!(text, ':') {
		Ok(
			MetaLine::Tags(text.split(',')
			                   .map(|split| split.trim().to_owned())
			                   .collect())
		)
	} else if strip_start_then_trim!(text, "tracking" | "Tracking" | "TRACKING")
		&& strip_start_then_trim!(text, ':') {
		
		if text.to_lowercase().as_str() == "always" {
			Ok(MetaLine::Tracking(TrackingSetting::Always))
		} else if text.to_lowercase().as_str() == "never" {
			Ok(MetaLine::Tracking(TrackingSetting::Never))
		} else {
			Err(anyhow!(
				"Invalid tracking setting in Node metadata: {text}\n\n\
				 Help: valid values are either `always` or `never`.\n\
				 At line nº{}, `{}`", source_line.line_number, source_line.text))
		}
	} else {
		Ok(MetaLine::Custom(text.to_string()))
	}
}

pub fn parse_metadata<'a>(lines: impl IntoIterator<Item = &'a UnparsedLine>) -> Result<NodeMetadata> {
	let meta_lines =
		lines.into_iter()
			 .map(|line| (line.line_number, parse_meta_line(line)));
	
	let mut title = None;
	let mut tags = vec![];
	let mut tracking = None;
	let mut customs = vec![];

	for (line_number, result) in meta_lines {
		let meta_line = result?;
		
		match meta_line {
			MetaLine::Title(title_to_set) => {
				match title {
					Some((old_number, old_title)) => {
						return Err(anyhow!(
							"Found double `node title` declaration.\n\
							 First: `{old_title}` at line nº{old_number}\n\
							 Second: `{title_to_set}` at line nº{line_number}\n\n\
							 Help: Delete one of the declarations.\n\
							 Help: Nodes cannot have more than one title."))
					},
					None => {
						title = Some((line_number, title_to_set))
					},
				}
			},
			MetaLine::Tags(tags_to_add) => {
				tags.extend(tags_to_add);
			},
			MetaLine::Tracking(tracking_to_set) => {
				match tracking {
					Some((old_number, old_tracking)) => {
						return Err(anyhow!(
							"Found double `tracking setting` declaration.\n\
							 First: `{old_tracking:?}` at line nº{old_number}\n\
							 Second: `{tracking:?}` at line nº{line_number}\n\n\
							 Help: Delete one of the declarations.\n\
							 Help: It doesn't make sense to set the same setting twice."))
					},
					None => {
						tracking = Some((line_number, tracking_to_set));
					},
				}
			},
			MetaLine::Custom(custom) => {
				customs.push(custom);
			},
		}
	}

	let Some((title_number, title_name)) = title
		else {
			return Err(anyhow!(
				"Missing `node title` declaration in node.\n\n\
				 Help: To declare a title, write a line with the syntax: `title: MyNodeTitleHere`\n\
				 Help: The title should be the first metadata line."))
		};
	
	let first_char = title_name.chars().next().unwrap();
	
	if !first_char.is_ascii_alphabetic() && first_char != '_' {
		return Err(anyhow!(
			"Invalid first character in `node title`.\n\
			 At line nº{title_number}, title: {title_name}\n\n\
			 Help: The first character of a title needs to be a ASCII letter or a underscore('_').\n\
			 Help: Titles cannot start with numbers or other special characters ('*', '/', '+', '-', ..)."))
	}
	
	if let Some(invalid_char) = 
		title_name
			.chars()
			.find(|ch| !ch.is_ascii_alphanumeric() && *ch != '_') {
		return Err(anyhow!(
				"Invalid character `{invalid_char}` in `node title`.\n\
				 Full Name: `{title_name}`
				 At line nº{title_number}\n\n\
				 Help: Titles can only contain letters, digits and underscores('_')."))
	}

	Ok(NodeMetadata {
		title: title_name,
		tags,
		tracking: tracking.map(|(_, t)| t),
		customs,
	})
}

#[test]
fn test_parsing() {
	macro_rules! assert_eq_ok {
	    ($str: literal, $pattern: expr) => {
		    pretty_assertions::assert_eq!(parse_meta_line(&UnparsedLine { line_number: 0, text: $str.to_string() }).unwrap(), $pattern)
	    };
	}
	
	use houtamelo_utils::prelude::*;
	
	assert_eq_ok!("title: Ch01_Awakening", MetaLine::Title(own!("Ch01_Awakening")));
	assert_eq_ok!("   tags: more, night", MetaLine::Tags(vec![own!("more"), own!("night")]));
	assert_eq_ok!("tags: day, light", MetaLine::Tags(vec![own!("day"), own!("light")]));
	assert_eq_ok!("\ttags: less, stuff", MetaLine::Tags(vec![own!("less"), own!("stuff")]));
	assert_eq_ok!("custom_tag: any info here", MetaLine::Custom(own!("custom_tag: any info here")));
	assert_eq_ok!("\tanother custom_tag: other info   ", MetaLine::Custom(own!("another custom_tag: other info")));
	assert_eq_ok!("tracking: always", MetaLine::Tracking(TrackingSetting::Always));

	assert_eq_ok!("tracking:  never", MetaLine::Tracking(TrackingSetting::Never));
	assert_eq_ok!("tracking:Never", MetaLine::Tracking(TrackingSetting::Never));
	assert_eq_ok!("tracking:\tNEVER", MetaLine::Tracking(TrackingSetting::Never));
	assert_eq_ok!("tracking:  Always", MetaLine::Tracking(TrackingSetting::Always));
	assert_eq_ok!("tracking:ALWAYS", MetaLine::Tracking(TrackingSetting::Always));
	assert_eq_ok!("tracking:NeVeR", MetaLine::Tracking(TrackingSetting::Never));
	assert_eq_ok!("tracking:AlWaYS", MetaLine::Tracking(TrackingSetting::Always));
	
	let valid_text = [
		"title: Ch01_Awakening",
		"   tags: more, night",
		"tags: day, light",
		"\ttags: less, stuff",
		"custom_tag: any info here",
		"\tanother custom_tag: other info   ",
		"tracking: always",
	];
	
	let unparsed_lines =
		valid_text
			.into_iter()
			.enumerate()
			.map(|(line_number, text)| UnparsedLine { line_number, text: text.to_string() })
			.collect::<Vec<UnparsedLine>>();

	let valid_meta = 
		parse_metadata(&unparsed_lines)
			.unwrap();
	
	assert_eq!(valid_meta, 
		NodeMetadata {
			title: own!("Ch01_Awakening"),
			tags: own_vec!["more", "night", "day", "light", "less", "stuff"],
			tracking: Some(TrackingSetting::Always),
			customs: own_vec!["custom_tag: any info here", "another custom_tag: other info"],
		}
	);
}