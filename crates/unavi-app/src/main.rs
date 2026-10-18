// Release builds get a windowed subsystem (no console); dev builds keep the
// console so logs are visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
#[cfg(target_family = "wasm")] use clap::CommandFactory;
use clap::Parser;
use tracing::Level;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Enable debug logging. Release binaries build `tracing`/`log` with
    /// `release_max_level_warn`, which compiles `debug!`/`info!` calls out
    /// entirely, so this only has an effect in dev builds.
    #[arg(long)]
    debug_log: bool,

    /// Keeps the identity key and document store in-memory.
    /// Useful for running multiple apps on the same machine.
    #[arg(long)]
    in_memory: bool,

    /// Runs in XR mode.
    #[cfg(not(target_family = "wasm"))]
    #[arg(long)]
    xr: bool,

    /// Enters this space namespace instead of the local home.
    #[arg(long)]
    join: Option<String>,
}

impl Args {
    /// XR is not a wasm arg: the web app has no `OpenXR` runtime to ask for.
    #[cfg_attr(
        target_family = "wasm",
        allow(
            clippy::unused_self,
            reason = "kept so both targets share one call site"
        )
    )]
    const fn xr(&self) -> bool {
        cfg_select! {
            target_family = "wasm" => false,
            _ => self.xr,
        }
    }
}

fn main() -> AppExit {
    let args = cfg_select! {
        target_family = "wasm" => parse_args_wasm(),
        _ => Args::parse(),
    };

    let log_level = if args.debug_log {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let mut app = App::new();

    // Don't panic on ECS errors in release builds.
    #[cfg(not(debug_assertions))]
    app.set_error_handler(bevy::ecs::error::warn);

    app.add_plugins(unavi_app::UnaviPlugin {
        in_memory: args.in_memory,
        xr: args.xr(),
        join: args.join,
        log_level,
    })
    .run()
}

/// Parses `Args` from the page's URL query string, ignoring keys `Args`
/// does not declare instead of failing the whole parse. A share link such as
/// `?join=<ns>&utm_source=x` must still join `<ns>`; one unrecognized
/// tracking param should not fall back to every default.
#[cfg(target_family = "wasm")]
fn parse_args_wasm() -> Args {
    let window = web_sys::window().expect("get window");
    let search = window.location().search().expect("get search");
    let params = web_sys::UrlSearchParams::new_with_str(&search).expect("parse search params");

    // `help` and `version` are real argument ids on the derived command, but
    // honoring them would print to a console that is not attached to
    // anything and then try to exit the process, which wasm cannot do.
    let command = Args::command();
    let known: std::collections::HashSet<&str> = command
        .get_arguments()
        .filter_map(clap::Arg::get_long)
        .filter(|long| *long != "help" && *long != "version")
        .collect();

    let mut argv = vec!["app".to_string()];

    for key in params.keys() {
        let Ok(key) = key else {
            continue;
        };
        let Some(key) = key.as_string() else {
            continue;
        };

        if !known.contains(key.as_str()) {
            web_sys::console::debug_1(&format!("ignoring unknown URL param: {key}").into());
            continue;
        }

        let Some(value) = params.get(&key) else {
            continue;
        };
        argv.push(format!("--{key}"));

        if !value.is_empty() {
            argv.push(value);
        }
    }

    if !argv.iter().any(|arg| arg == "--join")
        && let Ok(pathname) = window.location().pathname()
        && let Some(namespace) = space_from_path(&pathname)
    {
        argv.push("--join".to_string());
        argv.push(namespace.to_string());
    }

    web_sys::console::log_1(&format!("URL params: {:?}", &argv[1..]).into());

    match Args::try_parse_from(argv) {
        Ok(a) => a,
        Err(err) => {
            web_sys::console::warn_1(&format!("Error parsing params: {err}").into());
            Args::parse_from(["app"])
        }
    }
}

/// The namespace in a `/space/<namespace>` path, which is how a shared link
/// names a space on the web.
#[cfg(any(test, target_family = "wasm"))]
fn space_from_path(pathname: &str) -> Option<&str> {
    let rest = pathname.strip_prefix("/space/")?;
    let namespace = rest.split('/').next()?;
    (!namespace.is_empty()).then_some(namespace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_path_yields_its_namespace() {
        assert_eq!(space_from_path("/space/abc123"), Some("abc123"));
    }

    #[test]
    fn space_path_ignores_trailing_segments() {
        assert_eq!(space_from_path("/space/abc123/"), Some("abc123"));
        assert_eq!(space_from_path("/space/abc123/extra"), Some("abc123"));
    }

    #[test]
    fn non_space_paths_yield_nothing() {
        assert_eq!(space_from_path("/"), None);
        assert_eq!(space_from_path("/space/"), None);
        assert_eq!(space_from_path("/space"), None);
        assert_eq!(space_from_path("/spaces/abc"), None);
    }
}
