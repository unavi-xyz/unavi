use crate::{Args, Command};

pub fn process_args(args: &mut Args) {
    if args.path == ".unavi/server/<command>" {
        let folder = match args.command {
            Command::World { .. } => "world",
            Command::Social { .. } => "social",
            Command::All => "all",
        };

        args.path = format!(".unavi/server/{}", folder);
    }

    std::fs::create_dir_all(&args.path).unwrap();
}
