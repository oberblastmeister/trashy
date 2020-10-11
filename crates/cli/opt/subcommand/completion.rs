use std::io;

use clap::{Clap, IntoApp};
use clap_generate::generate;
use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};

#[derive(Debug, Clap)]
pub struct Opt {
    /// shell to generate copmletions for
    #[clap(possible_values = &["bash", "elvish", "fish", "powershell", "zsh"])]
    pub shell: String,
}

pub fn completion(opt: Opt) {
    // crate::Opt::clap()
    let mut app = crate::Opt::into_app();

    match &*opt.shell {
        "bash" => generate::<Bash, _>(&mut app, "trash", &mut io::stdout()),
        "fish" => generate::<Fish, _>(&mut app, "trash", &mut io::stdout()),
        "powershell" => generate::<PowerShell, _>(&mut app, "trash", &mut io::stdout()),
        "zsh" => generate::<Zsh, _>(&mut app, "trash", &mut io::stdout()),
        _ => panic!("wrong shell"),
    }
}
