use super::list;
use crate::{app, trash_item::MaybeIndexedTrashItems};
use anyhow::Result;

pub fn on_items_with_prompt(
    items: MaybeIndexedTrashItems,
    config_args: &app::ConfigArgs,
    action_name: &str,
    f: impl FnOnce(MaybeIndexedTrashItems) -> Result<()>,
) -> Result<()> {
    use dialoguer::Confirm;

    let len = items.len();
    let plural = if len == 1 { "s" } else { "" };
    println!("{len} item{plural} will be {action_name}");
    list::display_indexed_items(items.indexed_items(), config_args)?;
    if Confirm::new().with_prompt("Are you sure?").interact()? {
        f(items)?;
    }
    Ok(())
}
