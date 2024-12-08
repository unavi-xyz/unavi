use std::time::Duration;

use anyhow::Result;
use dwn::{stores::NativeDbStore, Dwn};
use tokio::task::JoinHandle;
use unavi_server::{Args, Command, StartOptions, Storage};

pub struct TestServer {
    pub domain_social: String,
    pub domain_world: String,
    pub task_social: JoinHandle<Result<()>>,
    pub task_world: JoinHandle<Result<()>>,
}

pub async fn setup_test_server() -> TestServer {
    let port_social = port_scanner::request_open_port().unwrap();
    let domain_social = local_domain(port_social);

    let port_world = port_scanner::request_open_port().unwrap();
    let domain_world = local_domain(port_world);

    let args_social = Args {
        debug: true,
        storage: Storage::Memory,
        command: Command::Social { port: port_social },
    };

    let args_world = Args {
        debug: true,
        storage: Storage::Memory,
        command: Command::World {
            domain: domain_world.clone(),
            port: port_world,
            remote_dwn: format!("http://{}", domain_social),
            threads: Some(1),
        },
    };

    let store = NativeDbStore::new_in_memory().unwrap();
    let dwn = Dwn::from(store);

    let opts = StartOptions::default();

    let task_social = tokio::spawn(unavi_server::start(args_social, opts.clone(), dwn.clone()));
    let task_world = tokio::spawn(unavi_server::start(args_world, opts, dwn));

    tokio::time::sleep(Duration::from_secs(5)).await;

    TestServer {
        domain_social,
        domain_world,
        task_social,
        task_world,
    }
}

fn local_domain(port: u16) -> String {
    format!("localhost:{}", port)
}
