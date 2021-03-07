use crate::db::get_connection;
use crate::UrlEncodedUrl;
use anyhow::anyhow;

pub async fn count_or_fail(url: UrlEncodedUrl) -> anyhow::Result<usize> {
    let mut conn = get_connection().await?;
    let domain_name = url
        .into_inner()
        .domain()
        .ok_or_else(|| anyhow!("Missing domain name in url"))?
        .to_string();

    let (count,): (i32,) =
        sqlx::query_as("SELECT COUNT(1) FROM link WHERE domainUrl = $1")
            .bind(domain_name)
            .fetch_one(&mut conn)
            .await?;

    Ok(count as usize)
}
