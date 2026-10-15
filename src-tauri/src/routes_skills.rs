//! Skills: markdown files under `~/.clipper/skills/`, mirrored into the index.
//!
//! The **filesystem is the source of truth** — the same rule as the vault. The
//! `skills` table is a queryable cache rebuilt from the directory on startup and
//! after every write, so a skill the user drops in by hand shows up too.

use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use axum::{
    extract::{Path as UrlPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{clock, db, frontmatter, server::AppState};

/// Bodies are capped so a runaway client cannot fill the disk.
const MAX_BODY_BYTES: usize = 256 * 1024;

/// Skill names are slugs. Rejecting separators and `..` is what keeps a name
/// from ever escaping the skills directory.
fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Rough size in tokens: ~4 characters per token.
fn token_estimate(body: &str) -> i64 {
    (body.chars().count() / 4) as i64
}

fn skill_path(cfg: &crate::config::Config, name: &str) -> PathBuf {
    cfg.skills_dir().join(format!("{name}.md"))
}

/// Markdown -> HTML, dropping embedded raw HTML.
///
/// Dropping the raw-HTML events is the server-side sanitize step: a `<script>`
/// in a skill body never reaches the output. The client still runs DOMPurify
/// over anything it puts in `x-html` — defence in depth, not either/or.
pub fn render_safe_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Event, Options, Parser};

    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let events = Parser::new_ext(markdown, options)
        .filter(|event| !matches!(event, Event::Html(_) | Event::InlineHtml(_)));

    let mut out = String::new();
    html::push_html(&mut out, events);
    out
}

/// Parse a skill file into a row.
fn read_skill_file(path: &Path, name: &str) -> io::Result<db::Skill> {
    let body = fs::read_to_string(path)?;
    let meta = frontmatter::parse(&body).unwrap_or_default();

    let last_modified = fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or_else(clock::now_unix);

    Ok(db::Skill {
        name: name.to_string(),
        source: meta.get("source").map(str::to_string).or(Some("local".into())),
        title: meta.get("title").map(str::to_string),
        description: meta.get("description").map(str::to_string),
        body_md: body,
        token_estimate: token_estimate(&fs::read_to_string(path)?),
        last_modified,
    })
}

/// Rebuild the `skills` table from the directory.
pub fn sync(cfg: &crate::config::Config, db: &db::Db) -> io::Result<usize> {
    let dir = cfg.skills_dir();
    fs::create_dir_all(&dir)?;

    let mut found = Vec::new();
    for entry in fs::read_dir(&dir)?.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if !valid_name(name) {
            continue;
        }
        match read_skill_file(&path, name) {
            Ok(skill) => found.push(skill),
            Err(e) => eprintln!("skills: {}: {e}", path.display()),
        }
    }

    let conn = db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let known: Vec<String> = conn
        .prepare("SELECT name FROM skills")
        .and_then(|mut st| {
            st.query_map([], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .unwrap_or_default();

    for skill in &found {
        db::upsert_skill(&conn, skill).map_err(io::Error::other)?;
    }
    // A file that is gone must not linger in the index.
    for name in known {
        if !found.iter().any(|skill| skill.name == name) {
            let _ = db::delete_skill(&conn, &name);
        }
    }

    Ok(found.len())
}

/// Router for the skills slice; merged into the main app.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/skills", get(list_skills))
        .route("/api/skills/rescan", post(rescan))
        .route(
            "/api/skills/{name}",
            get(get_skill).put(put_skill).delete(delete_skill),
        )
}

async fn list_skills(State(state): State<AppState>) -> Result<Json<Value>, crate::server::ApiError> {
    let skills = crate::server::with_db(&state, db::list_skills)?;
    Ok(Json(json!({ "skills": skills, "total": skills.len() })))
}

async fn get_skill(
    State(state): State<AppState>,
    UrlPath(name): UrlPath<String>,
) -> Result<Json<Value>, crate::server::ApiError> {
    if !valid_name(&name) {
        return Err(crate::server::ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid skill name",
        ));
    }

    match crate::server::with_db(&state, |conn| db::get_skill(conn, &name))? {
        Some(skill) => {
            let html = render_safe_html(&skill.body_md);
            let mut payload = json!(skill);
            payload["html"] = json!(html);
            Ok(Json(payload))
        }
        None => Err(crate::server::ApiError::new(
            StatusCode::NOT_FOUND,
            "no such skill",
        )),
    }
}

#[derive(Debug, Deserialize)]
struct SkillBody {
    body_md: String,
    source: Option<String>,
}

async fn put_skill(
    State(state): State<AppState>,
    UrlPath(name): UrlPath<String>,
    Json(body): Json<SkillBody>,
) -> Result<Json<Value>, crate::server::ApiError> {
    let invalid = |message: &str| {
        crate::server::ApiError::new(StatusCode::BAD_REQUEST, message.to_string())
    };

    if !valid_name(&name) {
        return Err(invalid("invalid skill name"));
    }
    if body.body_md.len() > MAX_BODY_BYTES {
        return Err(invalid("skill body is too large"));
    }

    let dir = state.cfg.skills_dir();
    fs::create_dir_all(&dir).map_err(crate::server::ApiError::internal)?;

    let body_md = match body.source.as_deref() {
        Some(source) if !source.trim().is_empty() => {
            // Keep the source in frontmatter so it survives a re-scan.
            if frontmatter::parse(&body.body_md).is_some() {
                body.body_md.clone()
            } else {
                format!("---\nsource: {source}\n---\n\n{}", body.body_md)
            }
        }
        _ => body.body_md.clone(),
    };

    // Atomic write: temp then rename.
    let target = skill_path(&state.cfg, &name);
    let tmp = target.with_extension("md.tmp");
    fs::write(&tmp, &body_md).map_err(crate::server::ApiError::internal)?;
    fs::rename(&tmp, &target).map_err(crate::server::ApiError::internal)?;

    let skill = read_skill_file(&target, &name).map_err(crate::server::ApiError::internal)?;
    crate::server::with_db(&state, |conn| db::upsert_skill(conn, &skill))?;

    Ok(Json(json!({
        "ok": true,
        "name": skill.name,
        "token_estimate": skill.token_estimate,
        "last_modified": skill.last_modified,
    })))
}

async fn delete_skill(
    State(state): State<AppState>,
    UrlPath(name): UrlPath<String>,
) -> Result<Json<Value>, crate::server::ApiError> {
    if !valid_name(&name) {
        return Err(crate::server::ApiError::new(
            StatusCode::BAD_REQUEST,
            "invalid skill name",
        ));
    }

    let target = skill_path(&state.cfg, &name);
    if target.exists() {
        fs::remove_file(&target).map_err(crate::server::ApiError::internal)?;
    }
    let existed = crate::server::with_db(&state, |conn| db::delete_skill(conn, &name))?;

    if existed || target.exists() {
        Ok(Json(json!({ "ok": true })))
    } else {
        Err(crate::server::ApiError::new(
            StatusCode::NOT_FOUND,
            "no such skill",
        ))
    }
}

async fn rescan(State(state): State<AppState>) -> Result<Json<Value>, crate::server::ApiError> {
    let cfg = state.cfg.clone();
    let handle = std::sync::Arc::clone(&state.db);
    let scanned = tokio::task::spawn_blocking(move || sync(&cfg, &handle))
        .await
        .map_err(crate::server::ApiError::internal)?
        .map_err(crate::server::ApiError::internal)?;
    Ok(Json(json!({ "ok": true, "scanned": scanned })))
}
