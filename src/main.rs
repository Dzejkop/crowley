use crowley::UrlEncodedUrl;
use warp::Filter;

const PORT: u16 = 3030;

#[tokio::main]
async fn main() {
    // Initialize from .env if it exists
    let _ = dotenv::dotenv();

    pretty_env_logger::init();

    let scrape = warp::post().and(
        warp::path!("scrape" / UrlEncodedUrl).and_then(crowley::routes::scrape),
    );
    let count =
        warp::path!("count" / UrlEncodedUrl).and_then(crowley::routes::count);
    let list =
        warp::path!("list" / UrlEncodedUrl).and_then(crowley::routes::list);

    let routes = scrape.or(count).or(list);

    log::info!("Crowley is running on port {}", PORT);
    warp::serve(routes).run(([127, 0, 0, 1], PORT)).await;
}
