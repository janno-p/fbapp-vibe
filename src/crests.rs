const DEFAULT_CREST_URL: &str = "/assets/default.svg";

pub fn find_crest_url(crest_url: Option<&str>) -> String {
    crest_url.unwrap_or(DEFAULT_CREST_URL).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_crest_url_returns_value() {
        const SOME_CREST_URL: &str = "https://crests.football-data.org/760.svg";
        assert_eq!(find_crest_url(Some(SOME_CREST_URL)), SOME_CREST_URL);
    }

    #[test]
    fn unknown_crest_url_falls_back_to_default() {
        assert_eq!(find_crest_url(None), DEFAULT_CREST_URL);
    }
}
