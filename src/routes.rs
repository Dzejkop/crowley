use crate::UrlEncodedUrl;
use hyper::StatusCode;
use std::convert::Infallible;

pub async fn scrape(
    url: UrlEncodedUrl,
) -> Result<impl warp::Reply, Infallible> {
    match crate::scrape::scrape_or_fail(url).await {
        Ok(_) => Ok(warp::reply::with_status("OK".to_string(), StatusCode::OK)),
        Err(err) => Ok(warp::reply::with_status(
            format!("{:?}", err),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

pub async fn count(url: UrlEncodedUrl) -> Result<impl warp::Reply, Infallible> {
    match crate::count::count_or_fail(url).await {
        Ok(num) => {
            Ok(warp::reply::with_status(format!("{}", num), StatusCode::OK))
        }
        Err(err) => Ok(warp::reply::with_status(
            format!("{:?}", err),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

pub async fn list(url: UrlEncodedUrl) -> Result<impl warp::Reply, Infallible> {
    let formatted_items =
        crate::list::list_or_fail(url).await.and_then(|items| {
            let formatted = serde_json::to_string(&items)?;

            Ok(formatted)
        });

    match formatted_items {
        Ok(formatted) => {
            Ok(warp::reply::with_status(formatted, StatusCode::OK))
        }
        Err(err) => Ok(warp::reply::with_status(
            format!("{:?}", err),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}
