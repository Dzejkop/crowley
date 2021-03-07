use std::collections::HashSet;

use cmd_lib::{run_cmd, run_fun, spawn};
use maplit::hashset;

// TODO: Replace cmd_lib with a custom spawning methods. Reroute stdout of child processes to internal buffers or files to clear test output and to be able to read when each process is ready
#[test]
fn acceptance() -> anyhow::Result<()> {
    run_cmd!(./setup.sh)?;
    let mut crowley = spawn!(cargo run)?;
    let mut page =
        spawn!(python3 -m http.server --directory ./tests/page 8080)?;

    // Wait for one second until the server and the scraper are both initialized
    // TODO: This is very hacky, and prone to failures if the initialization time gets any longer
    // This should be replaced by reading from stdout of both processes for some recognized pattern
    std::thread::sleep(std::time::Duration::from_secs(1));

    let actual_response = run_fun!(curl -X POST "http://127.0.0.1:3030/scrape/http%3A%2F%2Flocalhost%3A8080%2F")?;

    assert_eq!("OK", actual_response);

    let actual_count = run_fun!(curl -X GET "http://127.0.0.1:3030/count/http%3A%2F%2Flocalhost%3A8080%2F")?;
    let scraped_links = run_fun!(curl -X GET "http://127.0.0.1:3030/list/http%3A%2F%2Flocalhost%3A8080%2F")?;
    crowley.kill()?;
    page.kill()?;

    assert_eq!("5", actual_count);

    // skip '[' and ']'
    let scraped_links = &scraped_links[1..scraped_links.len() - 1];
    let scraped_links = scraped_links
        .split(',')
        .map(|link| link[1..link.len() - 1].to_string()) // Remove quotes
        .collect::<HashSet<_>>();

    let expected_links = hashset![
        "http://localhost:8080/".to_string(),
        "http://localhost:8080/page_2.html".to_string(),
        "http://localhost:8080/page_1.html".to_string(),
        "http://localhost:8080/something/test.html".to_string(),
        "http://localhost:8080/something".to_string(),
    ];
    assert_eq!(expected_links, scraped_links,);

    Ok(())
}
