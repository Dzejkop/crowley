use crate::db::get_connection;
use crate::UrlEncodedUrl;
use anyhow::anyhow;
use futures::future;
use hyper::header::CONTENT_TYPE;
use scraper::Selector;
use std::collections::HashSet;
use url::Url;

const BATCH_SIZE: usize = 128;

pub async fn scrape_or_fail(root_url: UrlEncodedUrl) -> anyhow::Result<()> {
    let root_url = root_url.into_inner();
    let domain_name = root_url
        .domain()
        .ok_or_else(|| anyhow!("Missing domain in url"))?
        .to_string();

    ensure_not_already_scraped(&domain_name).await?;

    log::info!("Scraping url {}, as domain {}", root_url, domain_name);

    let mut scraped_urls = HashSet::new();
    let mut remaining_urls = HashSet::new();
    remaining_urls.insert(root_url);
    loop {
        log::debug!("Running batch");
        let batch = take_from_set(&mut remaining_urls, BATCH_SIZE);

        for item in &batch {
            log::debug!("Scheduling {}", item);
            scraped_urls.insert(item.clone());
        }

        if batch.is_empty() {
            break;
        }

        // Scrape all the urls in the batch at once
        let batch_result = future::join_all(
            batch.into_iter().map(|url| scrape_url(url, &domain_name)),
        )
        .await;

        let mut all: HashSet<_> = HashSet::new();
        for item in batch_result {
            let item = item?;
            for item in item {
                all.insert(item);
            }
        }

        for item in all {
            if !scraped_urls.contains(&item) {
                remaining_urls.insert(item);
            }
        }
    }

    log::info!("Done scraping {}", domain_name);

    save_links_to_db(&domain_name, scraped_urls).await?;

    log::info!("Saved scraped data for {} to database", domain_name);

    Ok(())
}

async fn ensure_not_already_scraped(domain_name: &str) -> anyhow::Result<()> {
    let mut conn = get_connection().await?;

    let url: Option<(String,)> =
        sqlx::query_as("SELECT url FROM domain WHERE url = $1")
            .bind(domain_name)
            .fetch_optional(&mut conn)
            .await?;

    if let Some((url,)) = url {
        return Err(anyhow!(
            "Cannot scrape {}, it has already been scraped",
            url
        ));
    }

    Ok(())
}

async fn save_links_to_db(
    domain_name: &str,
    scraped_urls: HashSet<Url>,
) -> anyhow::Result<()> {
    let mut conn = get_connection().await?;

    sqlx::query("INSERT INTO domain (url) VALUES ($1)")
        .bind(&domain_name)
        .execute(&mut conn)
        .await?;

    // Would be better to insert all at once, but https://github.com/launchbadge/sqlx/issues/294 must first be resolved
    for link in scraped_urls {
        sqlx::query("INSERT INTO link (url, domainUrl) VALUES ($1, $2)")
            .bind(link.as_str())
            .bind(&domain_name)
            .execute(&mut conn)
            .await?;
    }

    Ok(())
}

fn take_from_set(set: &mut HashSet<Url>, n: usize) -> HashSet<Url> {
    let mut drain = set.drain();

    let mut ret = HashSet::with_capacity(n);

    for _ in 0..n {
        if let Some(item) = drain.next() {
            ret.insert(item);
        } else {
            break;
        }
    }

    *set = drain.collect();

    ret
}

async fn scrape_url(
    url: Url,
    domain_name: &str,
) -> anyhow::Result<HashSet<Url>> {
    let document_response = reqwest::get(url.clone()).await?;
    if let Some(content_type) = document_response.headers().get(CONTENT_TYPE) {
        let content_type = content_type.to_str()?;
        if !content_type.contains("text/html") {
            return Ok(HashSet::new());
        }
    } else {
        return Ok(HashSet::new());
    }

    let document = document_response.text().await?;

    scrape_document(&url, &document, domain_name)
}

fn scrape_document(
    base_url: &Url,
    document: &str,
    domain_name: &str,
) -> anyhow::Result<HashSet<Url>> {
    let document = scraper::Html::parse_document(&document);
    let selector =
        Selector::parse("a").map_err(|err| anyhow::anyhow!("{:?}", err))?;
    let mut unique_urls = HashSet::new();
    for elem in document.select(&selector) {
        if let Some(href) = elem.value().attr("href") {
            let url = if href.starts_with("/") {
                let mut href_url = base_url.clone();
                href_url.set_path(&resolve_path(href_url.path(), href));
                href_url
            } else if let Ok(href_url) = href.parse::<Url>() {
                href_url
            } else {
                continue;
            };

            if url.domain() == Some(domain_name) {
                unique_urls.insert(url);
            }
        }
    }

    Ok(unique_urls)
}

fn resolve_path(abs: &str, rel: &str) -> String {
    let mut base_segments: Vec<_> =
        abs.split('/').filter(|s| !s.is_empty()).collect();
    let mut rel_segments = rel.split('/').filter(|s| !s.is_empty()).peekable();

    if rel_segments.peek().is_none() {
        return abs.to_string();
    }

    while let Some(next) = rel_segments.next() {
        if next == ".." {
            base_segments.remove(base_segments.len() - 1);
        } else {
            base_segments.push(next);
        }
    }

    base_segments.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use maplit::hashset;
    use test_case::test_case;

    const EXAMPLE_DOCUMENT: &str = indoc! {r##"
        <!DOCTYPE html>
        <html>
        <body>

        <h1>My First Heading</h1>

        <p>My first paragraph.</p>
        <div class="whatever">
            <a href="#">Empty link</a>
            <a href="/link">Relative link</a>
            <a href="http://www.domain.com/whatever">Absolute link</a>
        </div>

        </body>
        </html>
    "##};

    #[test_case("here", "there" => "here/there")]
    #[test_case("here/and_here", "there" => "here/and_here/there")]
    #[test_case("here/and_there/", "there" => "here/and_there/there")]
    #[test_case("here/and_there", "there/and_here" => "here/and_there/there/and_here")]
    #[test_case("here/and_there", "../and_back/and_here" => "here/and_back/and_here")]
    #[test_case("here/and_there", "../and_back/../and_other_here" => "here/and_other_here")]
    fn resolve_path_returns(a: &str, b: &str) -> &'static str {
        let x = resolve_path(a, b).into_boxed_str();

        Box::leak(x)
    }

    #[test]
    fn scrape_document_returns() {
        let base_url = "http://www.domain.com/here".parse().unwrap();
        let scraped_urls =
            scrape_document(&base_url, EXAMPLE_DOCUMENT, "www.domain.com")
                .unwrap();

        let expected = hashset! {
            "http://www.domain.com/whatever".parse().unwrap(),
            "http://www.domain.com/here/link".parse().unwrap(),
        };

        assert_eq!(expected, scraped_urls);
    }
}
