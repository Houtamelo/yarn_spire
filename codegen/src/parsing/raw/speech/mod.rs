#[cfg(test)]
mod tests;

use crate::expressions::yarn_expr::YarnExpr;
use crate::parsing::macros::{return_if_err, starts_with_any, strip_start_then_trim};
use crate::parsing::raw::{Content, ParseRawYarn};
use crate::{expressions, LineNumber};
use anyhow::{anyhow, Result};
use expressions::parse_yarn_expr;
use genco::lang::Rust;
use genco::prelude::{quoted, FormatInto};
use genco::{quote_in, Tokens};
use houtamelo_utils::prelude::None;
use std::iter::Peekable;
use std::mem;
use std::str::Chars;
use trim_in_place::TrimInPlace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Speaker {
	Literal(String),
	Variable(String),
}

impl FormatInto<Rust> for &Speaker {
	fn format_into(self, tokens: &mut Tokens<Rust>) {
		match self {
			Speaker::Literal(literal) =>
				quote_in!(*tokens => $(quoted(literal))),
			Speaker::Variable(var_name) =>
				quote_in!(*tokens => storage.get_var::<$var_name>()),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Speech {
	pub line_number: LineNumber,
	pub line_id: Option<String>,
	pub speaker: Option<Speaker>,
	pub text: (String, Vec<YarnExpr>),
	pub tags: Vec<String>,
}

enum CharState {
	Std,
	StringLiteral,
	StringLiteralIgnoreNext,
}

enum State {
	Lit {
		ignore_next: bool,
	},
	Arg {
		char_state: CharState,
		previous_char: char,
		nesting: Vec<char>,
		sum: String,
	},
}

fn parse_line(chars: &mut Peekable<Chars>, line_number: LineNumber) -> Result<Speech> {
	let mut state = State::Lit { ignore_next: false };
	let mut speaker = None;
	let mut literal = String::new();
	let mut args: Vec<String> = vec![];
	let mut metadata = None;

	while let Some(next) = chars.next() {
		match &mut state {
			State::Lit { ignore_next: ignore_next @ true } => {
				*ignore_next = false;
				literal.push(next);
			}
			State::Lit { ignore_next: ignore_next @ false } => {
				match next {
					'\\' => {
						*ignore_next = true;
					}
					'{' => {
						state = State::Arg {
							char_state: CharState::Std,
							previous_char: next,
							nesting: vec![],
							sum: String::new(),
						};

						literal.push_str("{}");
					}
					'}' => {
						return Err(anyhow!(
							"Unexpected closing delimiter `}}` when parsing literal.\n\
							 Built so far: \n\
							 \tLiteral: `{literal}`\n\
							 \tArguments: `{args:?}`\n\n\
							 Help: The closing delimiter `}}` does not match any opening delimiter `{{`.\n\
							 Help: If you want to use '{{', '}}' inside a string literal, escape it with a backslash (`\\`)."));
					}
					'#' => {
						let built_metadata =
							std::iter::once('#')
								.chain(chars.by_ref())
								.collect::<String>();

						if !built_metadata.is_empty() {
							metadata = Some(built_metadata)
						}

						break;
					}
					':' => {
						if speaker.is_none()
							&& !literal.is_empty()
							&& literal.trim().chars().none(char::is_whitespace)
							&& args.is_empty() {
							let mut speaker_str = mem::take(&mut literal);
							speaker_str.trim_in_place();
							speaker = Some(Speaker::Literal(speaker_str));
						} else if speaker.is_none()
							&& literal.trim() == "{}"
							&& args.len() == 1
						{
							literal.clear();

							let unparsed_speaker =
								args.remove(0);
							let expr =
								parse_yarn_expr(&unparsed_speaker)
									.map_err(|err| anyhow!(
										"Could not parse `speaker variable` as `YarnExpr`.\n\
										 Error: {err:?}\n\
										 Unparsed: `{unparsed_speaker}`\n\
										 Built so far: \n\
										 \tLiteral: `{literal}`\n\
										 \tArguments: `{args:?}`\n")
									)?;

							let YarnExpr::GetVar(speaker_var_name) = expr
							else {
								return Err(anyhow!(
										"Invalid `speaker variable` argument.\n\
										 Expected expression of type `YarnExpr::VarGet(var_name)`,\
										 Got: {expr:?}\n\
										 Unparsed: `{unparsed_speaker}`\n\
										 Built so far: \n\
										 \tLiteral: `{literal}`\n\
										 \tArguments: `{args:?}`\n\
										 \n\
										 Help: the speaker argument can only be a string literal(`John`) \
										 or a variable name(`{{$variable_name}}`)."));
							};

							speaker = Some(Speaker::Variable(speaker_var_name));
						} else {
							literal.push(next);
						}
					}
					other => {
						literal.push(other);
					}
				}
			}
			State::Arg {
				char_state: char_state @ CharState::Std, previous_char,
				nesting, sum
			} => {
				match next {
					'"' => {
						*char_state = CharState::StringLiteral;
						*previous_char = '"';
						sum.push('"');
					}
					nest @ ('(' | '{' | '[') => {
						*previous_char = nest;
						nesting.push(nest);
						sum.push(nest);
					}
					un_nest @ (')' | '}' | ']') =>
						if let Some(nest) = nesting.pop() {
							if matches!((nest, un_nest), ('(', ')') | ('{', '}') | ('[', ']')) {
								*previous_char = un_nest;
								sum.push(un_nest);
							} else {
								return Err(anyhow!(
									"Invalid closing delimiter `{un_nest}` when parsing argument.\n\
									 Argument: `{sum}`\n\
									 Nesting: `{nesting:?}`\n\
									 Built so far: \n\
									 \tLiteral: `{literal}`\n\
									 \tArguments: `{args:?}`\n\
									 \n\
									 Help: the closing delimiter `{un_nest}` does not match the most-recent opening delimiter `{nest}`.\n\
									 Help: if you want to use '{{', '}}' inside a string literal, escape it with a backslash (`\\`)."));
							}
						} else if un_nest == '}' {
							args.push(mem::take(sum));
							state = State::Lit { ignore_next: false };
						} else {
							return Err(anyhow!(
								"Unexpected closing delimiter `{un_nest}` when parsing argument.\n\
								 Argument: `{sum}`\n\
								 Nesting: `{nesting:?}`\n\
								 Built so far: \n\
								 \tLiteral: `{literal}`\n\
								 \tArguments: `{args:?}`\n\
								 \n\
								 Help: if you want to use '{{', '}}' inside a string literal, escape it with a backslash (`\\`)."));
						},
					other => {
						*previous_char = other;
						sum.push(other);
					}
				}
			}
			State::Arg {
				char_state: char_state @ CharState::StringLiteral,
				previous_char, sum, nesting: _nesting
			} => {
				match next {
					'"' => {
						*char_state = CharState::Std;
						*previous_char = '"';
						sum.push('"');
					}
					'\\' => {
						*char_state = CharState::StringLiteralIgnoreNext;
					}
					other => {
						*previous_char = other;
						sum.push(other);
					}
				}
			}
			State::Arg {
				char_state: char_state @ CharState::StringLiteralIgnoreNext,
				previous_char, sum, nesting: _nesting
			} => {
				*char_state = CharState::StringLiteral;
				*previous_char = next;
				sum.push(next);
			}
		}
	}

	match state {
		State::Lit { ignore_next } => {
			if ignore_next {
				return Err(anyhow!(
					"Speech ended with an escape character (`\\`).\n\
					 Built so far: \n\
					 \tLiteral: `{literal}`\n\
					 \tArguments: `{args:?}`\n\n\
					 Help: The escape character(`\\`) means nothing if there's no character after it."));
			}
		}
		State::Arg {
			char_state: _char_state, previous_char: _previous_char,
			nesting, sum
		} => {
			return Err(anyhow!(
				"Speech ended with an open delimiter (building an argument).\n\
				 Argument: `{sum}`\n\
				 Nesting: `{nesting:?}`\n\
				 Built so far: \n\
				 \tLiteral: `{literal}`\n\
				 \tArguments: `{args:?}`\n\n\
				 Help: The argument `{sum}` is not closed.\n\
				 Help: For every opening delimiter(`(`, `{{`, `[`), there must be a matching closing delimiter(`)`, `}}`, `]`).\n\
				 Help: If you want to use '{{', '}}' inside a string literal, escape it with a backslash (`\\`)."));
		}
	}

	let args_expr =
		build_args(args.clone())
			.map_err(|err| anyhow!(
				"Could not parse argument as `YarnExpr`.\n\
				 Error: `{err:?}`\n\
				 Speaker: `{speaker:?}`\n\
		         Literal: `{literal}`\n\
		         Metadata: `{metadata:?}`")
			)?;

	literal.trim_in_place();

	if literal.is_empty()
		&& args_expr.is_empty() {
		return Err(anyhow!(
			"Both literal and arguments are empty.\n\
			 Built so far: \n\
			 \tLiteral: `{literal}`\n\
			 \tArguments: `{args:?}`\n"));
	}

	let Some(after_hash) = metadata
	else {
		return Ok(Speech {
			line_number,
			line_id: None,
			speaker,
			text: (literal, args_expr),
			tags: vec![],
		})
	};

	let mut tags: Vec<String> =
		after_hash
			.split('#')
			.filter_map(|tag| {
				let trimmed = tag.trim();
				if trimmed.is_empty() {
					None
				} else {
					Some(trimmed.to_string())
				}
			}).collect();

	let line_id: Vec<String> =
		tags.extract_if(|tag| {
			let mut temp = tag.as_str();
			if strip_start_then_trim!(temp, "line")
				&& strip_start_then_trim!(temp, ":") {
				*tag = temp.to_string();
				true
			} else {
				false
			}
		}).collect();

	match line_id.len() {
		0 => Ok(Speech {
			line_number,
			line_id: None,
			speaker,
			text: (literal, args_expr),
			tags,
		}),
		1 => Ok(Speech {
			line_number,
			line_id: line_id.into_iter().next(),
			speaker,
			text: (literal, args_expr),
			tags,
		}),
		_ => Err(anyhow!(
			"More than one `line_id` tag found.\n\
			 Ids found: `{}`\n\
			 Tags: `{}`\n\
			 Built so far: \n\
			 \tLiteral: `{literal}`\n\
			 \tArguments: `{args:?}`\n"
			, line_id.join(", "), tags.join(", "))),
	}
}

fn build_args(unparsed_args: Vec<String>) -> Result<Vec<YarnExpr>> {
	let exprs = unparsed_args
		.iter()
		.map(|unparsed_str| parse_yarn_expr(unparsed_str)
			.map_err(|err| anyhow!("{err:?}\nAll Unparsed Arguments: `{unparsed_args:?}`")))
		.try_collect()?;

	Ok(exprs)
}

impl ParseRawYarn for Speech {
	fn parse_raw_yarn(line: &str, line_number: LineNumber) -> Option<Result<Content>> {
		let line = line.trim();

		if starts_with_any!(line, "<<" | "->" | "<-") {
			return None;
		}

		let mut chars = line.chars().peekable();

		let speech = return_if_err!(parse_line(&mut chars, line_number)
			.map_err(|err|anyhow!(
				"Could not parse line as `Speech`.\n\
				 Remaining line: `{}`\n\
				 Error: `{err:?}`", chars.collect::<String>())));

		Some(Ok(Content::Speech(speech)))
	}
}