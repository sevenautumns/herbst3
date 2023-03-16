use std::str::FromStr;

use anyhow::{Context, Result};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use strum::EnumString;

use crate::herbstclient::get_layout;

#[derive(Parser)]
#[grammar_inline = r#"
node        = {
    "(" ~ ((type | id | layout | node) ~ WHITE_SPACE*)+ ~ ")"
}
type        = { "split" | "clients" }
layout      = { layout_type ~ ":" ~ (layout_size ~ ":"*)+ }
layout_type = { "vertical" | "horizontal" | "max" | "grid" }
layout_size = { (ASCII_DIGIT | ".")+ }
id          = { "0x" ~ ASCII_HEX_DIGIT+ }
"#]
struct LayoutParser;

#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum LayoutType {
    Vertical,
    Horizontal,
    Max,
    Grid,
}

pub fn get_layout_stack(index: &[u8]) -> Result<Vec<LayoutType>> {
    let mut layouts = Vec::new();
    let layout = get_layout()?;
    let mut node = LayoutParser::parse(Rule::node, &layout)?
        .next()
        .context("")?;

    for i in index {
        layouts.push(get_layout_type(&node)?);

        let mut node_iter = node.into_inner().filter(|p| p.as_rule() == Rule::node);
        node = node_iter.next().context("")?;
        if 1.eq(i) {
            node = node_iter.next().context("")?;
        }
    }

    Ok(layouts)
}

fn get_layout_type(node: &Pair<Rule>) -> Result<LayoutType> {
    let layout = node
        .clone()
        .into_inner()
        .find(|p| p.as_rule() == Rule::layout)
        .context("")?
        .into_inner()
        .find(|p| p.as_rule() == Rule::layout_type)
        .context("")?
        .as_str();
    Ok(LayoutType::from_str(layout)?)
}
