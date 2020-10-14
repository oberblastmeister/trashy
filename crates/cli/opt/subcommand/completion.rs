use std::io;

use clap::{ArgEnum, Clap, IntoApp};
use clap_generate::generate;
use clap_generate::generators::{Bash, Elvish, Fish, PowerShell, Zsh};

#[derive(Debug, Copy, Clone, ArgEnum)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    Powershell,
    Zsh,
}

#[derive(Debug, Clap)]
pub struct Opt {
    /// shell to generate copmletions for
    #[clap(arg_enum)]
    #[clap(case_insensitive = true)]
    pub shell: Shell,
}

pub fn completion(opt: Opt) {
    let mut app = crate::Opt::into_app();

    match opt.shell {
        Shell::Bash => generate::<Bash, _>(&mut app, "trash", &mut io::stdout()),
        Shell::Fish => generate::<Fish, _>(&mut app, "trash", &mut io::stdout()),
        Shell::Powershell => generate::<PowerShell, _>(&mut app, "trash", &mut io::stdout()),
        Shell::Zsh => generate::<Zsh, _>(&mut app, "trash", &mut io::stdout()),
        Shell::Elvish => generate::<Elvish, _>(&mut app, "trash", &mut io::stdout()),
    }
}
