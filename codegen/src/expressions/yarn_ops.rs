use anyhow::{Result, anyhow};
use genco::lang::Rust;
use genco::prelude::FormatInto;
use genco::{quote_in, Tokens};
use crate::expressions::{SynBinOp, SynUnaryOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YarnUnaryOp {
	Not,
	Negate,
}

impl YarnUnaryOp {
	#[allow(unused)]
	pub(super) fn resolve(self) -> &'static str {
		match self {
			YarnUnaryOp::Not => "!",
			YarnUnaryOp::Negate => "-",
		}
	}
	
	pub(super) fn try_from_syn(unary_syn: SynUnaryOp) -> Result<Self> {
		match unary_syn {
			SynUnaryOp::Not(_) => Ok(YarnUnaryOp::Not),
			SynUnaryOp::Neg(_) => Ok(YarnUnaryOp::Negate),
			invalid_op => {
				Err(anyhow!("Invalid unary operator: {invalid_op:?}"))
			},
		}
	}
}


impl FormatInto<Rust> for &YarnUnaryOp {
	fn format_into(self, tokens: &mut Tokens<Rust>) {
		match self {
			YarnUnaryOp::Not => {
				quote_in!(*tokens => !);
			},
			YarnUnaryOp::Negate => {
				quote_in!(*tokens => -);
			},
		}
	}
}


#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YarnBinaryOp {
	Add, Sub,
	Mul,
	Div, Rem,
	And, Or,
	Eq, Ne,
	Lt, Le,
	Gt, Ge,
	BitXor, BitAnd, BitOr,
}

impl YarnBinaryOp {
	pub(super) fn try_from_syn(syn_op: SynBinOp) -> Result<Self> {
		match syn_op {
			SynBinOp::Add(_) => Ok(YarnBinaryOp::Add),
			SynBinOp::Sub(_) => Ok(YarnBinaryOp::Sub),
			SynBinOp::Mul(_) => Ok(YarnBinaryOp::Mul), 
			SynBinOp::Div(_) => Ok(YarnBinaryOp::Div),
			SynBinOp::Rem(_) => Ok(YarnBinaryOp::Rem),
			SynBinOp::And(_) => Ok(YarnBinaryOp::And),
			SynBinOp::Or (_) => Ok(YarnBinaryOp::Or),
			SynBinOp::Eq (_) => Ok(YarnBinaryOp::Eq), 
			SynBinOp::Ne (_) => Ok(YarnBinaryOp::Ne),
			SynBinOp::Lt (_) => Ok(YarnBinaryOp::Lt), 
			SynBinOp::Le (_) => Ok(YarnBinaryOp::Le),
			SynBinOp::Gt (_) => Ok(YarnBinaryOp::Gt), 
			SynBinOp::Ge (_) => Ok(YarnBinaryOp::Ge),
			SynBinOp::BitAnd(_) => Ok(YarnBinaryOp::BitAnd),
			SynBinOp::BitOr (_) => Ok(YarnBinaryOp::BitOr),
			SynBinOp::BitXor(_) => Ok(YarnBinaryOp::BitXor),
			invalid_op => {
				Err(anyhow!("Invalid binary operator: {invalid_op:?}"))
			},
		}
	}
}

impl FormatInto<Rust> for &YarnBinaryOp {
	fn format_into(self, tokens: &mut Tokens<Rust>) {
		match self {
			YarnBinaryOp::Add => {
				quote_in!(*tokens => +);
			},
			YarnBinaryOp::Sub => {
				quote_in!(*tokens => -);
			},
			YarnBinaryOp::Mul => {
				quote_in!(*tokens => *);
			},
			YarnBinaryOp::Div => {
				quote_in!(*tokens => /);
			},
			YarnBinaryOp::Rem => {
				quote_in!(*tokens => %);
			},
			YarnBinaryOp::And => {
				quote_in!(*tokens => &&);
			},
			YarnBinaryOp::Or => {
				quote_in!(*tokens => ||);
			},
			YarnBinaryOp::Eq => {
				quote_in!(*tokens => ==);
			},
			YarnBinaryOp::Ne => {
				quote_in!(*tokens => !=);
			},
			YarnBinaryOp::Lt => {
				quote_in!(*tokens => <);
			},
			YarnBinaryOp::Le => {
				quote_in!(*tokens => <=);
			},
			YarnBinaryOp::Gt => {
				quote_in!(*tokens => >);
			},
			YarnBinaryOp::Ge => {
				quote_in!(*tokens => >=);
			},
			YarnBinaryOp::BitXor => {
				quote_in!(*tokens => ^);
			},
			YarnBinaryOp::BitAnd => {
				quote_in!(*tokens => &);
			},
			YarnBinaryOp::BitOr => {
				quote_in!(*tokens => |);
			},
		}
	}
}
