use std::time::Duration;

use tracing_test::traced_test;
use unavi_server::{Args, Command, Storage};

fn local_domain(port: u16) -> String {
    format!("localhost:{}", port)
}

#[tokio::test]
#[traced_test]
async fn test_world_host() {
    let port_social = port_scanner::request_open_port().unwrap();
    let domain_social = local_domain(port_social);

    let port_world = port_scanner::request_open_port().unwrap();
    let domain_world = local_domain(port_world);

    let args_social = Args {
        debug: true,
        command: Command::Social {
            domain: domain_social.clone(),
            path: String::new(),
            port: port_social,
            storage: Storage::Memory,
        },
    };

    let args_world = Args {
        debug: true,
        command: Command::World {
            domain: domain_world,
            dwn_url: format!("http://{}", domain_social),
            path: String::new(),
            port: port_world,
            storage: Storage::Memory,
        },
    };

    let social_task = tokio::spawn(unavi_server::start(args_social));
    let world_task = tokio::spawn(unavi_server::start(args_world));
    tokio::time::sleep(Duration::from_secs(5)).await;

    assert!(logs_contain("Sync successful."));

    social_task.abort();
    world_task.abort();
}
