use xshell::cwd;

#[macro_export]
macro_rules! put_test {
    ($name:ident, $path:expr, $dir:expr, file:expr) => {
        #[test]
        fn $name() {
            let name = 
            let (dir, cmd) = crate::util::setup(stringify!($name));

            $fun(dir, cmd);

            if cfg!(feature = "pcre2") {
                let (dir, cmd) = crate::util::setup_pcre2(stringify!($name));
                $fun(dir, cmd);
            }
        }
    };

    ($name:ident, $path:expr, file:expr) => {
        #[test]
        fn $name() -> Result<()> {
            use crate::utils;
            use crate::trash_lib::put;

            let mut dir = utils::get_dir()?;
            dir.push("/tests");
            dir.push($file);
            put(&[dir])?;
        }
    }
}
