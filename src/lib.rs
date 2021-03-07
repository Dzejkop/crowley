pub mod count;
pub mod db;
pub mod list;
pub mod routes;
pub mod scrape;
pub mod url_encoded;

pub type UrlEncodedUrl = url_encoded::UrlEncoded<url::Url>;
