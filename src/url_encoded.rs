use std::str::FromStr;

#[derive(Debug)]
pub struct UrlEncoded<T>(T);

impl<T> UrlEncoded<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromStr for UrlEncoded<T>
where
    T: FromStr,
    <T as FromStr>::Err: Send + Sync + std::error::Error + 'static,
{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = urlencoding::decode(s)?;

        Ok(Self(decoded.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("http%3A%2F%2Flocalhost%3A8080%2F" => "http://localhost:8080/")]
    #[test_case("String%20with%20spaces" => "String with spaces")]
    #[test_case("https%3A%2F%2Fgithub.com%2Flaunchbadge%2Fsqlx%2Fissues%2F294" => "https://github.com/launchbadge/sqlx/issues/294")]
    fn url_encoded_parse(s: &str) -> &'static str {
        let url: UrlEncoded<String> = s.parse().unwrap();

        Box::leak(url.into_inner().into_boxed_str())
    }
}
