#![windows_subsystem = "windows"]

use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tiny_http::{Header, Method, Response, Server, StatusCode};

#[derive(Deserialize)]
struct BuildEntry {
    path: String,
}

#[derive(Deserialize)]
struct ProjectRecord {
    name: String,
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
        .unwrap_or(8080);
    let bind = get_arg_value(&args, "--bind").unwrap_or_else(|| "0.0.0.0".to_owned());
    let host = get_arg_value(&args, "--host").unwrap_or_else(|| bind.clone());

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

    let site_dir = match build_site(project) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Failed to build site: {err}");
            std::process::exit(1);
        }
    };

    let address = format!("{}:{}", bind, port);
    let server = match Server::http(&address) {
        Ok(server) => {
            println!("Bridge status: connected");
            println!("--- NETWORK CONFIGURATION ---");
            println!("Binding Address: {}", address);
            if !host.is_empty() && host != "0.0.0.0" {
                println!("Hotspot link: http://{}:{}/", host, port);
            }
            println!("Local PC link: http://127.0.0.1:{}/", port);
            println!("-----------------------------");
            println!("Keep this window open while downloading.");
            server
        },
        Err(err) => {
            eprintln!("CRITICAL ERROR: Failed to bind server on {}. Port might be in use.", address);
            eprintln!("Error details: {err}");
            std::process::exit(1);
        }
    };

    for request in server.incoming_requests() {
        if request.method() == &Method::Options {
            let response = with_cors(Response::from_string("")
                .with_status_code(StatusCode(204)));
            let _ = request.respond(response);
            continue;
        }
        let url = request.url().to_owned();
        let path = split_url(&url);

        if path == "/" {
            let _ = request.respond(with_cors(serve_index(&site_dir)));
        } else if let Some(path) = path.strip_prefix("/files/") {
            let _ = request.respond(with_cors(serve_file(&site_dir, path)));
        } else {
            let _ = request.respond(with_cors(Response::from_string("Not found").with_status_code(StatusCode(404))));
        }
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

fn build_site(project: &ProjectRecord) -> Result<PathBuf, String> {
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
    if let Some(extra) = &project.added_file
        && let Some(item) = copy_artifact(&project.main_path, extra, &files_dir) {
            items.push(item);
    }

    let index_path = base_dir.join("index.html");
    let mut index = fs::File::create(&index_path).map_err(|err| err.to_string())?;
    let html = build_index_html(&project.name, &items);
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
        .map(sanitize_segment)?;
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

fn build_index_html(project_name: &str, items: &[ArtifactItem]) -> String {
    let mut html = String::new();
    html.push_str("<!doctype html><html><head><meta charset=\"utf-8\">");
    html.push_str("<title>BuildBridge Serve</title>");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    html.push_str("<style>body{font-family:Arial,sans-serif;margin:24px;background-color:#f5f5f5;}\n");
    html.push_str(".card{background:white;padding:16px;border-radius:8px;box-shadow:0 2px 4px rgba(0,0,0,0.1);}\n");
    html.push_str(".item{margin-bottom:12px;padding:8px;border-bottom:1px solid #eee;}\n");
    html.push_str("a{color:#1565c0;text-decoration:none;font-weight:bold;font-size:1.1em;}\n");
    html.push_str("small{color:#666;}</style>");
    html.push_str("</head><body><div class=\"card\">");
    html.push_str(&format!("<h2>{}</h2>", escape_html(project_name)));
    if items.is_empty() {
        html.push_str("<p>No artifacts available.</p>");
    } else {
        html.push_str("<div>");
        for item in items {
            html.push_str("<div class=\"item\">");
            let link = format!("files/{}", item.file_name);
            html.push_str(&format!(
                "<a href=\"{}\" download>{}</a><br><small>Size: {} bytes</small>",
                escape_html(&link),
                escape_html(&item.label),
                item.size
            ));
            html.push_str("</div>");
        }
        html.push_str("</div>");
    }
    html.push_str("</div></body></html>");
    html
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn split_url(url: &str) -> &str {
    url.split_once('?').map(|(path, _)| path).unwrap_or(url)
}

fn with_cors<T: std::io::Read>(response: Response<T>) -> Response<T> {
    response
        .with_header(cors_header("Access-Control-Allow-Origin", "*"))
        .with_header(cors_header(
            "Access-Control-Allow-Methods",
            "GET, OPTIONS",
        ))
        .with_header(cors_header(
            "Access-Control-Allow-Headers",
            "Content-Type",
        ))
}

fn cors_header(name: &str, value: &str) -> Header {
    Header::from_bytes(name, value).unwrap_or_else(|_| {
        Header::from_bytes("Access-Control-Allow-Origin", "*")
            .expect("static CORS header")
    })
}

fn serve_index(site_dir: &Path) -> Response<Box<dyn std::io::Read + Send>> {
    let index_path = site_dir.join("index.html");
    if let Ok(file) = fs::File::open(&index_path) {
        let boxed: Box<dyn std::io::Read + Send> = Box::new(file);
        Response::new(
            StatusCode(200),
            vec![Header::from_bytes("Content-Type", "text/html").unwrap()],
            boxed,
            None,
            None,
        )
    } else {
        Response::from_string("Index not found").with_status_code(StatusCode(404)).boxed()
    }
}

fn serve_file(site_dir: &Path, file_name: &str) -> Response<Box<dyn std::io::Read + Send>> {
    let safe_name = sanitize_segment(file_name);
    let file_path = site_dir.join("files").join(safe_name);
    if let Ok(file) = fs::File::open(&file_path) {
        let content_type = if file_path.extension().and_then(|e| e.to_str()) == Some("apk") {
            "application/vnd.android.package-archive"
        } else {
            "application/octet-stream"
        };
        let boxed: Box<dyn std::io::Read + Send> = Box::new(file);
        Response::new(
            StatusCode(200),
            vec![Header::from_bytes("Content-Type", content_type).unwrap()],
            boxed,
            None,
            None,
        )
    } else {
        Response::from_string("File not found").with_status_code(StatusCode(404)).boxed()
    }
}
