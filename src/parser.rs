use std::str::FromStr;

use anyhow::{Context, Result};
use log::debug;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use strum::{Display, EnumString};

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

#[derive(Debug, PartialEq, EnumString, Display)]
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

    debug!("Layout stack: {:?}", &layouts);
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
