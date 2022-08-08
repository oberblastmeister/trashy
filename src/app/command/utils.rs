use super::list;
use crate::{app, range_set::RangeSet, range_syntax, trash_item::MaybeIndexedTrashItems};
use anyhow::Result;
use clap::{ArgAction, Parser};

pub fn on_items_with_prompt(
    items: MaybeIndexedTrashItems,
    config_args: &app::ConfigArgs,
    action_name: &str,
    f: impl FnOnce(MaybeIndexedTrashItems) -> Result<()>,
) -> Result<()> {
    use dialoguer::Confirm;

    let len = items.len();
    let plural = if len == 1 { "" } else { "s" };
    println!("{len} item{plural} will be {action_name}");
    list::display_indexed_items(items.indexed_items(), config_args)?;
    if Confirm::new().with_prompt("Are you sure?").interact()? {
        f(items)?;
    }
    Ok(())
}

#[derive(Debug, Parser)]
pub struct Force {
    /// Skip confirmation
    ///
    /// By default, 'trashy' will ask for confirmation before restoring or permanently removing files.
    /// You can opt out of this by adding '--force'.
    /// This can be useful in scripts.
    #[clap(short, long)]
    pub force: bool,
}

#[derive(Debug, Parser)]
pub struct Ranges {
    /// Filter by ranges
    ///
    /// This should be used to restore specific items.
    /// To find the ranges, look at the 'i' column in the table shown by 'trash list'.
    /// The option is called '--ranges' but you can also use individual indices by just typing out the number.
    /// Ranges are zero based, inclusive at the start, and exclusive at the end, just like rust ranges.
    /// Examples:
    ///     --ranges='1 5..9 10..12'
    ///     --ranges='1 4 5 6'
    #[clap(
       short,
       long,
       conflicts_with_all = list::QueryArgs::CONFLICTS,
       action = ArgAction::Append,
       verbatim_doc_comment
    )]
    pub ranges: Vec<String>,
}

impl Ranges {
    pub fn parse(&self) -> Result<RangeSet> {
        self.ranges.iter().flat_map(|s| range_syntax::parse_ranges(s)).collect()
    }
}
