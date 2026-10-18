//! Fetch a meaningful title for a URL when the extractor's title is garbled.
//!
//! Uses public oEmbed endpoints where available (no API key required) so we
//! can recover the real video/article title from platforms that hide it from
//! the page's `<title>` tag.
//!
//! Currently supported: YouTube, Vimeo, SoundCloud, Twitter. Falls back to
//! scanning the page HTML for `og:title` for everyone else.

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct FetchedTitle {
    pub title: String,
    pub author: Option<String>,
    pub source: &'static str,
}

const HTTP_TIMEOUT: Duration = Duration::from_secs(8);

pub async fn fetch_title(url: &str) -> Option<FetchedTitle> {
    if let Some(host) = extract_host(url) {
        if host.contains("youtube.com") || host.contains("youtu.be") {
            return fetch_oembed("https://www.youtube.com/oembed", url, "YouTube").await;
        }
        if host.contains("vimeo.com") {
            return fetch_oembed("https://vimeo.com/api/oembed.json", url, "Vimeo").await;
        }
        if host.contains("soundcloud.com") {
            return fetch_oembed(
                "https://soundcloud.com/oembed",
                url,
                "SoundCloud",
            )
            .await;
        }
        if host.contains("twitter.com") || host.contains("x.com") {
            return fetch_oembed(
                "https://publish.twitter.com/oembed",
                url,
                "Twitter",
            )
            .await;
        }
    }
    fetch_og_title(url).await
}

fn extract_host(url: &str) -> Option<String> {
    url.split("://").nth(1)?.split('/').next().map(|s| s.to_lowercase())
}

async fn fetch_oembed(
    endpoint: &str,
    target: &str,
    source: &'static str,
) -> Option<FetchedTitle> {
    let client = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .user_agent("Clipper/0.9 (+https://github.com/aswathas/clipper)")
        .build()
        .ok()?;
    let resp = client
        .get(endpoint)
        .query(&[("url", target), ("format", "json")])
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().await.ok()?;
    let title = v
        .get("title")
        .and_then(|t| t.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())?;
    let author = v
        .get("author_name")
        .and_then(|t| t.as_str())
        .map(|s| s.trim().to_string());
    Some(FetchedTitle { title, author, source })
}

async fn fetch_og_title(url: &str) -> Option<FetchedTitle> {
    let client = reqwest::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .user_agent("Clipper/0.9 (+https://github.com/aswathas/clipper)")
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .ok()?;
    let resp = client.get(url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body = resp.text().await.ok()?;
    let title = extract_meta(&body, "og:title")
        .or_else(|| extract_meta(&body, "twitter:title"))
        .or_else(|| extract_title(&body))?;
    Some(FetchedTitle {
        title: title.trim().to_string(),
        author: None,
        source: "og",
    })
}

fn extract_meta(html: &str, property: &str) -> Option<String> {
    let needle1 = format!("property=\"{property}\"");
    let needle2 = format!("name=\"{property}\"");
    for needle in [&needle1, &needle2] {
        if let Some(idx) = html.find(needle.as_str()) {
            let after = &html[idx + needle.len()..];
            if let Some(content_idx) = after.find("content=\"") {
                let start = content_idx + "content=\"".len();
                let rest = &after[start..];
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].to_string());
                }
            }
        }
    }
    None
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let open = lower.find("<title")?;
    let after = &html[open..];
    let gt = after.find('>')?;
    let start = &after[gt + 1..];
    let close = start.find("</title>")?;
    Some(start[..close].to_string())
}

/// Blocking variant for the drainer. Falls through to `None` on any failure.
pub fn fetch_title_blocking(url: &str) -> Option<FetchedTitle> {
    let host = extract_host(url)?;
    let endpoint: &str = if host.contains("youtube.com") || host.contains("youtu.be") {
        "https://www.youtube.com/oembed"
    } else if host.contains("vimeo.com") {
        "https://vimeo.com/api/oembed.json"
    } else if host.contains("soundcloud.com") {
        "https://soundcloud.com/oembed"
    } else if host.contains("twitter.com") || host.contains("x.com") {
        "https://publish.twitter.com/oembed"
    } else {
        return fetch_og_title_blocking(url);
    };
    let client = reqwest::blocking::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .user_agent("Clipper/0.9 (+https://github.com/aswathas/clipper)")
        .build()
        .ok()?;
    let resp = client
        .get(endpoint)
        .query(&[("url", url), ("format", "json")])
        .send()
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: serde_json::Value = resp.json().ok()?;
    let title = v
        .get("title")
        .and_then(|t| t.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())?;
    let author = v
        .get("author_name")
        .and_then(|t| t.as_str())
        .map(|s| s.trim().to_string());
    Some(FetchedTitle {
        title,
        author,
        source: if endpoint.contains("youtube") {
            "YouTube"
        } else if endpoint.contains("vimeo") {
            "Vimeo"
        } else if endpoint.contains("soundcloud") {
            "SoundCloud"
        } else {
            "Twitter"
        },
    })
}

fn fetch_og_title_blocking(url: &str) -> Option<FetchedTitle> {
    let client = reqwest::blocking::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .user_agent("Clipper/0.9 (+https://github.com/aswathas/clipper)")
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .ok()?;
    let resp = client.get(url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body = resp.text().ok()?;
    let title = extract_meta(&body, "og:title")
        .or_else(|| extract_meta(&body, "twitter:title"))
        .or_else(|| extract_title(&body))?;
    Some(FetchedTitle {
        title: title.trim().to_string(),
        author: None,
        source: "og",
    })
}
