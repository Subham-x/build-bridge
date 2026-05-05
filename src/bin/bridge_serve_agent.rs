use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tiny_http::{Response, Server, StatusCode};

#[derive(Deserialize)]
struct BuildEntry {
    name: String,
    path: String,
}

#[derive(Deserialize)]
struct ProjectRecord {
    name: String,
    #[serde(rename = "type")]
    project_type: String,
    main_path: String,
    builds: Vec<BuildEntry>,
    #[serde(rename = "added-file", default)]
    added_file: Option<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let projects_path = get_arg_value(&args, "--projects").unwrap_or_default();
    let project_name = get_arg_value(&args, "--project").unwrap_or_default();
    let port: u16 = get_arg_value(&args, "--port")
        .and_then(|value| value.parse().ok())
        .unwrap_or(4000);
    let token = get_arg_value(&args, "--token");

    if projects_path.is_empty() || project_name.is_empty() {
        eprintln!("Missing --projects or --project argument.");
        std::process::exit(1);
    }

    let projects = match load_projects(&projects_path) {
        Ok(projects) => projects,
        Err(err) => {
            eprintln!("Failed to read projects: {err}");
            std::process::exit(1);
        }
    };

    let project = match projects.iter().find(|project| project.name == project_name) {
        Some(project) => project,
        None => {
            eprintln!("Project '{project_name}' not found.");
            std::process::exit(1);
        }
    };

    let site_dir = match build_site(project, token.as_deref()) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Failed to build site: {err}");
            std::process::exit(1);
        }
    };

    let address = format!("127.0.0.1:{port}");
    let server = match Server::http(&address) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("Failed to bind server on {address}: {err}");
            std::process::exit(1);
        }
    };

    let base_url = if let Some(token) = token.as_deref() {
        format!("http://{address}/?token={token}")
    } else {
        format!("http://{address}/")
    };
    println!("Bridge status: connected");
    println!("Listening on {base_url}");

    for request in server.incoming_requests() {
        let url = request.url();
        let (path, query) = split_url(url);

        if let Some(required) = token.as_deref() {
            let token_ok = query
                .and_then(|query| find_query_param(query, "token"))
                .map_or(false, |value| value == required);
            if !token_ok {
                let response = Response::from_string("Forbidden")
                    .with_status_code(StatusCode(403));
                let _ = request.respond(response);
                continue;
            }
        }

        let response = if path == "/" {
            serve_index(&site_dir)
        } else if let Some(path) = path.strip_prefix("/files/") {
            serve_file(&site_dir, path)
        } else {
            Response::from_string("Not found").with_status_code(StatusCode(404))
        };

        let _ = request.respond(response);
    }
}

fn get_arg_value(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == key)
        .and_then(|index| args.get(index + 1))
        .map(|value| value.to_owned())
}

fn load_projects(path: &str) -> Result<Vec<ProjectRecord>, String> {
    let data = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&data).map_err(|err| err.to_string())
}

fn build_site(project: &ProjectRecord, token: Option<&str>) -> Result<PathBuf, String> {
    let base_dir = env::temp_dir()
        .join("buildbridge-serve")
        .join(sanitize_segment(&project.name));
    let files_dir = base_dir.join("files");
    fs::create_dir_all(&files_dir).map_err(|err| err.to_string())?;

    let mut items = Vec::new();
    for build in &project.builds {
        if let Some(item) = copy_artifact(&project.main_path, &build.path, &files_dir) {
            items.push(item);
        }
    }
    if let Some(extra) = &project.added_file {
        if let Some(item) = copy_artifact(&project.main_path, extra, &files_dir) {
            items.push(item);
        }
    }

    let index_path = base_dir.join("index.html");
    let mut index = fs::File::create(&index_path).map_err(|err| err.to_string())?;
    let html = build_index_html(&project.name, &items, token);
    index.write_all(html.as_bytes()).map_err(|err| err.to_string())?;

    Ok(base_dir)
}

fn copy_artifact(main_path: &str, file_path: &str, files_dir: &Path) -> Option<ArtifactItem> {
    let path = Path::new(file_path);
    let source = if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(main_path).join(path)
    };
    if !source.exists() {
        eprintln!("Missing file: {}", source.display());
        return None;
    }

    let file_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| sanitize_segment(name))?;
    let dest = unique_destination(files_dir, &file_name);

    if let Err(err) = fs::copy(&source, &dest) {
        eprintln!("Failed to copy {}: {err}", source.display());
        return None;
    }

    let size = source.metadata().map(|meta| meta.len()).unwrap_or(0);
    Some(ArtifactItem {
        label: file_name,
        file_name: dest
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned(),
        size,
    })
}

fn unique_destination(base: &Path, file_name: &str) -> PathBuf {
    let mut candidate = base.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    let ext = Path::new(file_name).extension().and_then(|ext| ext.to_str());

    for index in 1..1000 {
        let name = match ext {
            Some(ext) => format!("{stem}-{index}.{ext}"),
            None => format!("{stem}-{index}"),
        };
        candidate = base.join(name);
        if !candidate.exists() {
            return candidate;
        }
    }

    candidate
}

fn sanitize_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

struct ArtifactItem {
    label: String,
    file_name: String,
    size: u64,
}

fn build_index_html(project_name: &str, items: &[ArtifactItem], token: Option<&str>) -> String {
    let mut html = String::new();
    html.push_str("<!doctype html><html><head><meta charset=\"utf-8\">");
    html.push_str("<title>BuildBridge Serve</title>");
    html.push_str("<style>body{font-family:Arial,sans-serif;margin:24px;}\n");
    html.push_str(".item{margin-bottom:12px;}a{color:#1565c0;}</style>");
    html.push_str("</head><body>");
    html.push_str(&format!("<h2>{}</h2>", escape_html(project_name)));
    if items.is_empty() {
        html.push_str("<p>No artifacts available.</p>");
    } else {
        html.push_str("<ul>");
        for item in items {
            html.push_str("<li class=\"item\">");
            let link = if let Some(token) = token {
                format!("files/{}?token={}", item.file_name, token)
            } else {
                format!("files/{}", item.file_name)
            };
            html.push_str(&format!(
                "<a href=\"{}\" download>{}</a> <small>({} bytes)</small>",
                escape_html(&link),
                escape_html(&item.label),
                item.size
            ));
            html.push_str("</li>");
        }
        html.push_str("</ul>");
    }
    html.push_str("</body></html>");
    html
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn split_url(url: &str) -> (&str, Option<&str>) {
    if let Some((path, query)) = url.split_once('?') {
        (path, Some(query))
    } else {
        (url, None)
    }
}

fn find_query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    for pair in query.split('&') {
        let mut iter = pair.splitn(2, '=');
        if let Some(name) = iter.next() {
            if name == key {
                return iter.next();
            }
        }
    }
    None
}

fn serve_index(site_dir: &Path) -> Response<std::io::Cursor<Vec<u8>>> {
    let index_path = site_dir.join("index.html");
    if let Ok(mut file) = fs::File::open(&index_path) {
        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_ok() {
            return Response::from_data(buffer);
        }
    }
    Response::from_string("Index not found").with_status_code(StatusCode(404))
}

fn serve_file(site_dir: &Path, file_name: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    let safe_name = sanitize_segment(file_name);
    let file_path = site_dir.join("files").join(safe_name);
    if let Ok(mut file) = fs::File::open(&file_path) {
        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_ok() {
            return Response::from_data(buffer);
        }
    }
    Response::from_string("File not found").with_status_code(StatusCode(404))
}
