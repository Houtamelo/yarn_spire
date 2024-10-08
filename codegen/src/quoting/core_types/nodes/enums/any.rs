// This file isn't used anymore, but it's here for reference.

/*
use crate::config::YarnConfig;
use crate::quoting::quotable_types::enums;
use crate::quoting::quotable_types::enums::LineEnum;
use crate::quoting::quotable_types::node::{IDNode, LinesMap};
use crate::quoting::util::SeparatedItems;
use genco::prelude::rust::Tokens;
use genco::quote;

fn tokens_imports(cfg: &YarnConfig) -> Tokens {
	quote! {
		#![allow(non_camel_case_types)]
		#![allow(non_snake_case)]
		#![allow(unused)]
		
		use houtamelo_utils::prelude::*;
		use serde::{Deserialize, Serialize};
		use $(&cfg.shared_qualified)::*;
	}
}

pub fn all_tokens(
	cfg: &YarnConfig,
	node: &IDNode,
	lines_map: &LinesMap,
) -> Option<Tokens> {
	if lines_map.speeches.is_empty()
		&& lines_map.commands.is_empty()
		&& lines_map.options_forks.is_empty() {
		return None;
	}

	let enum_name = enums::enum_type_any(&node.metadata.title);

	let tokens_imports = tokens_imports(cfg);
	let tokens_enum = tokens_enum(lines_map, &enum_name);
	let tokens_from_impl = tokens_from_impl(lines_map, &enum_name);

	Some(quote! {
		$tokens_imports
		$tokens_enum
		$tokens_from_impl
	})
}

fn tokens_enum(lines_map: &LinesMap, enum_name: &str) -> Tokens {
	let enum_variants = lines_map
		.speeches
		.iter()
		.map(|(_, line_enum)| line_enum)
		.chain(
			lines_map.commands
			         .iter()
			         .map(|(_, line_enum)| line_enum))
		.chain(
			lines_map.options_forks
			         .iter()
			         .map(|(_, line_enum)| line_enum))
		.map(LineEnum::variant_name);

	quote! {
		#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
		pub enum $enum_name {
			$(SeparatedItems(enum_variants, ",\n"))
		}
	}
}

fn tokens_from_impl(lines_map: &LinesMap, enum_name: &str) -> Tokens {
	let from_speech = lines_map
		.speeches
		.iter()
		.map(|(_, line_enum)|
			quote! { 
				$(line_enum.any_qualified()) => {
					Instruction::Speech($(line_enum.typed_qualified()).into())
				}, 
			});

	let from_command = lines_map
		.commands
		.iter()
		.map(|(_, line_enum)|
			quote! { 
				$(line_enum.any_qualified()) => {
					Instruction::Command($(line_enum.typed_qualified()).into())
				}, 
			});

	let from_options_fork = lines_map
		.options_forks
		.iter()
		.map(|(_, line_enum)|
			quote! { 
				$(line_enum.any_qualified()) => {
					Instruction::Options($(line_enum.typed_qualified()).into())
				}, 
			});

	quote! {
		impl From<$enum_name> for Instruction {
			fn from(value: $enum_name) -> Self {
				match value {
					$(SeparatedItems(from_speech, "\n"))
					$(SeparatedItems(from_command, "\n"))
					$(SeparatedItems(from_options_fork, "\n"))
				}
			}
		}
	}
}
*/

