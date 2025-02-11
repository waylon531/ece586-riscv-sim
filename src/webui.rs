use std::net::TcpListener;
use tinyhttp::prelude::*;

// serve the main web app page
#[get("/")]
fn get(_req: Request) -> Response {
    Response::new()
        .status_line("HTTP/1.1 200 OK\r\n")
        .mime("text/plain")
        .body(include_bytes!("webui/app.html").to_vec())
}


pub fn run_server() {
  let socket = TcpListener::bind(":::9001").unwrap();
  let routes = Routes::new(vec![get()]);
  let config = Config::new().routes(routes);
  let http = HttpListener::new(socket, config);

  println!("Web UI is listening on port 9001");

  http.start();
}

