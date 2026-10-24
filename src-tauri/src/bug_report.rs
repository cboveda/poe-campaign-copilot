//! Bug reporting opens a fixed issue form with no local diagnostics in its URL.
//! Users can review/redact files via Open logs and attach them themselves.

const REPORT_BUG_BASE_URL: &str =
    "https://github.com/cboveda/poe-campaign-copilot/issues/new?template=bug_report.yml";

pub fn bug_report_url() -> &'static str {
    REPORT_BUG_BASE_URL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_url_is_a_fixed_form_with_no_diagnostic_parameters() {
        assert_eq!(
            bug_report_url(),
            "https://github.com/cboveda/poe-campaign-copilot/issues/new?template=bug_report.yml"
        );
        assert!(!bug_report_url().contains("details="));
        assert!(!bug_report_url().contains("body="));
    }
}
