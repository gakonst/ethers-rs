use ethers_core::types::{Address, Chain};
use std::env::VarError;

#[derive(Debug, thiserror::Error)]
pub enum EtherscanError {
    #[error("Chain {0} not supported")]
    ChainNotSupported(Chain),
    #[error("Contract execution call failed: {0}")]
    ExecutionFailed(String),
    #[error("Balance failed")]
    BalanceFailed,
    #[error("Transaction receipt failed")]
    TransactionReceiptFailed,
    #[error("Gas estimation failed")]
    GasEstimationFailed,
    #[error("Bad status code: {0}")]
    BadStatusCode(String),
    #[error(transparent)]
    EnvVarNotFound(#[from] VarError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("Contract source code not verified: {0}")]
    ContractCodeNotVerified(Address),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Local networks (e.g. anvil, ganache, geth --dev) cannot be indexed by etherscan")]
    LocalNetworksNotSupported,
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("Missing field: {0}")]
    Builder(String),
    #[error("Missing solc version: {0}")]
    MissingSolcVersion(String),
    #[error("Invalid API Key")]
    InvalidApiKey,
    #[error("Sorry, you have been blocked by Cloudflare, See also https://community.cloudflare.com/t/sorry-you-have-been-blocked/110790")]
    BlockedByCloudflare,
}

/// etherscan/polyscan is protected by cloudflare, which can lead to html responses like `Sorry, you have been blocked` See also <https://community.cloudflare.com/t/sorry-you-have-been-blocked/110790>
///
/// This returns true if the `txt` is a cloudflare error response
pub(crate) fn is_blocked_by_cloudflare_response(txt: &str) -> bool {
    txt.to_lowercase().contains("sorry, you have been blocked")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_response() {
        let resp = "<!DOCTYPE html>\n<!--[if lt IE 7]> <html class=\"no-js ie6 oldie\" lang=\"en-US\"> <![endif]-->\n<!--[if IE 7]>    <html class=\"no-js ie7 oldie\" lang=\"en-US\"> <![endif]-->\n<!--[if IE 8]>    <html class=\"no-js ie8 oldie\" lang=\"en-US\"> <![endif]-->\n<!--[if gt IE 8]><!--> <html class=\"no-js\" lang=\"en-US\"> <!--<![endif]-->\n<head>\n<title>Attention Required! | Cloudflare</title>\n<meta charset=\"UTF-8\" />\n<meta http-equiv=\"Content-Type\" content=\"text/html; charset=UTF-8\" />\n<meta http-equiv=\"X-UA-Compatible\" content=\"IE=Edge\" />\n<meta name=\"robots\" content=\"noindex, nofollow\" />\n<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\" />\n<link rel=\"stylesheet\" id=\"cf_styles-css\" href=\"/cdn-cgi/styles/cf.errors.css\" />\n<!--[if lt IE 9]><link rel=\"stylesheet\" id='cf_styles-ie-css' href=\"/cdn-cgi/styles/cf.errors.ie.css\" /><![endif]-->\n<style>body{margin:0;padding:0}</style>\n\n\n<!--[if gte IE 10]><!-->\n<script>\n  if (!navigator.cookieEnabled) {\n    window.addEventListener('DOMContentLoaded', function () {\n      var cookieEl = document.getElementById('cookie-alert');\n      cookieEl.style.display = 'block';\n    })\n  }\n</script>\n<!--<![endif]-->\n\n\n</head>\n<body>\n  <div id=\"cf-wrapper\">\n    <div class=\"cf-alert cf-alert-error cf-cookie-error\" id=\"cookie-alert\" data-translate=\"enable_cookies\">Please enable cookies.</div>\n    <div id=\"cf-error-details\" class=\"cf-error-details-wrapper\">\n      <div class=\"cf-wrapper cf-header cf-error-overview\">\n        <h1 data-translate=\"block_headline\">Sorry, you have been blocked</h1>\n        <h2 class=\"cf-subheadline\"><span data-translate=\"unable_to_access\">You are unable to access</span> polygonscan.com</h2>\n      </div><!-- /.header -->\n\n      <div class=\"cf-section cf-highlight\">\n        <div class=\"cf-wrapper\">\n          <div class=\"cf-screenshot-container cf-screenshot-full\">\n            \n              <span class=\"cf-no-screenshot error\"></span>\n            \n          </div>\n        </div>\n      </div><!-- /.captcha-container -->\n\n      <div class=\"cf-section cf-wrapper\">\n        <div class=\"cf-columns two\">\n          <div class=\"cf-column\">\n            <h2 data-translate=\"blocked_why_headline\">Why have I been blocked?</h2>\n\n            <p data-translate=\"blocked_why_detail\">This website is using a security service to protect itself from online attacks. The action you just performed triggered the security solution. There are several actions that could trigger this block including submitting a certain word or phrase, a SQL command or malformed data.</p>\n          </div>\n\n          <div class=\"cf-column\">\n            <h2 data-translate=\"blocked_resolve_headline\">What can I do to resolve this?</h2>\n\n            <p data-translate=\"blocked_resolve_detail\">You can email the site owner to let them know you were blocked. Please include what you were doing when this page came up and the Cloudflare Ray ID found at the bottom of this page.</p>\n          </div>\n        </div>\n      </div><!-- /.section -->\n\n      <div class=\"cf-error-footer cf-wrapper w-240 lg:w-full py-10 sm:py-4 sm:px-8 mx-auto text-center sm:text-left border-solid border-0 border-t border-gray-300\">\n  <p class=\"text-13\">\n    <span class=\"cf-footer-item sm:block sm:mb-1\">Cloudflare Ray ID: <strong class=\"font-semibold\">74d2aa5ed9e27367</strong></span>\n    <span class=\"cf-footer-separator sm:hidden\">&bull;</span>\n    <span id=\"cf-footer-item-ip\" class=\"cf-footer-item hidden sm:block sm:mb-1\">\n      Your IP:\n      <button type=\"button\" id=\"cf-footer-ip-reveal\" class=\"cf-footer-ip-reveal-btn\">Click to reveal</button>\n      <span class=\"hidden\" id=\"cf-footer-ip\">62.96.232.178</span>\n      <span class=\"cf-footer-separator sm:hidden\">&bull;</span>\n    </span>\n    <span class=\"cf-footer-item sm:block sm:mb-1\"><span>Performance &amp; security by</span> <a rel=\"noopener noreferrer\" href=\"https://www.cloudflare.com/5xx-error-landing\" id=\"brand_link\" target=\"_blank\">Cloudflare</a></span>\n    \n  </p>\n  <script>(function(){function d(){var b=a.getElementById(\"cf-footer-item-ip\"),c=a.getElementById(\"cf-footer-ip-reveal\");b&&\"classList\"in b&&(b.classList.remove(\"hidden\"),c.addEventListener(\"click\",function(){c.classList.add(\"hidden\");a.getElementById(\"cf-footer-ip\").classList.remove(\"hidden\")}))}var a=document;document.addEventListener&&a.addEventListener(\"DOMContentLoaded\",d)})();</script>\n</div><!-- /.error-footer -->\n\n\n    </div><!-- /#cf-error-details -->\n  </div><!-- /#cf-wrapper -->\n\n  <script>\n  window._cf_translation = {};\n  \n  \n</script>\n\n</body>\n</html>\n";

        assert!(is_blocked_by_cloudflare_response(resp));
    }
}
