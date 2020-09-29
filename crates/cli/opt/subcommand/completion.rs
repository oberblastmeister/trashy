use std::io;

use structopt::clap::Shell;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// shell to generate copmletions for
    #[structopt(possible_values = &Shell::variants())]
    pub shell: Shell,
}

pub fn completion(opt: Opt) {
    crate::Opt::clap().gen_completions_to(env!("CARGO_PKG_NAME"), opt.shell, &mut io::stdout());
}
