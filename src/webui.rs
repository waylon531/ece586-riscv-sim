use std::net::TcpListener;
use tinyhttp::prelude::*;

// serve the static resources for the web ui
#[get("/:")]
fn get_assets(req: Request) -> Response {
    let wildcard = req.get_wildcard().unwrap();
    let (code, mime_type, file): (u16, &str, &[u8]) = match wildcard.as_str() {
        "/css/bootstrap.min.css"    => (200, "text/css", include_bytes!("webui/css/bootstrap.min.css")),
        "/js/bootstrap.min.js"    => (200, "text/javascript", include_bytes!("webui/js/bootstrap.min.js")),
        "/"             => (200, "text/html", include_bytes!("webui/index.html")),
        _               => (404, "text/plain", b"404"),
    };

    Response::new()
        .status_line(format!("HTTP/1.1 {} OK\r\n", code))
        .mime(mime_type)
        .body(file.to_vec())
}
 
// Example 1: Can return anything that implements Into<Vec<u8>>
#[get("/api")] dfgdfgdfg
fn get_api() -> &'static str {
  "Hello World!"
}

pub fn run_server() {
  let socket = TcpListener::bind(":::9001").unwrap();
  let routes = Routes::new(vec![get_assets(), get_api()]);
  let config = Config::new().routes(routes);
  let http = HttpListener::new(socket, config);

  println!("Web UI is listening on port 9001");

  http.start();
}
