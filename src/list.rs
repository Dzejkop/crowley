use crate::db::get_connection;
use crate::UrlEncodedUrl;
use anyhow::anyhow;

pub async fn list_or_fail(url: UrlEncodedUrl) -> anyhow::Result<Vec<String>> {
    let mut conn = get_connection().await?;
    let domain_name = url
        .into_inner()
        .domain()
        .ok_or_else(|| anyhow!("Missing domain name in url"))?
        .to_string();

    let urls: Vec<(String,)> =
        sqlx::query_as("SELECT url FROM link WHERE domainUrl = $1")
            .bind(domain_name)
            .fetch_all(&mut conn)
            .await?;

    let urls = urls.into_iter().map(|x| x.0).collect();

    Ok(urls)
}
