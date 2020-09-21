mod list;

use structopt::StructOpt;

use list::ListOpts;

#[derive(StructOpt)]
enum SubCommand {
    List(ListOpts),
}
