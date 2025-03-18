use std::net::TcpListener;
use tinyhttp::prelude::*;
use std::{default, env};
use std::io::Write;
use std::process::{Command, Stdio};
use std::fs;
use crossbeam_channel::Sender as CbSender;
use single_value_channel::Receiver as SvcReceiver;
use std::sync::{LazyLock,Mutex};
use crate::register::Register;
use url_encoded_data::UrlEncodedData;
use crate::statetransfer::{self, ControlCode, MachineState};
use serde_json::json;
static COMMANDS_TX: LazyLock<Mutex<Option<CbSender<ControlCode>>>> = LazyLock::new(|| Mutex::new(None));
static STATE_RX: LazyLock<Mutex<Option<SvcReceiver<MachineState>>>> = LazyLock::new(|| Mutex::new(None));
//static mut state : LazyLock<Mutex<Option<MachineState>>> = std::sync::LazyLock::new(||Mutex::new(Some(MachineState::empty())));

// serve the static resources for the web ui
// these are baked into the executable
#[get("/:")]
fn get_assets(req: Request) -> Response {
    let wildcard = req.get_wildcard().unwrap();
    let (code, mime_type, file): (u16, &str, &[u8]) = match wildcard.as_str() {
        "/css/bootstrap.min.css"    => (200, "text/css", include_bytes!("webui/css/bootstrap.min.css")),
        "/css/codemirror.css"    => (200, "text/css", include_bytes!("webui/css/codemirror.css")),
        "/css/extra.css"    => (200, "text/css", include_bytes!("webui/css/extra.css")),
        "/js/bootstrap.min.js"    => (200, "text/javascript", include_bytes!("webui/js/bootstrap.min.js")),
        "/js/codemirror.js"    => (200, "text/javascript", include_bytes!("webui/js/codemirror.js")),
        "/js/extra.js"    => (200, "text/javascript", include_bytes!("webui/js/extra.js")),
        "/"             => (200, "text/html", include_bytes!("webui/index.html")),
        _               => (404, "text/plain", b"404"),
    };

    Response::new()
    .status_line(format!("HTTP/1.1 {} {}\r\n", code, match code { 404 => "Not Found", _ => "OK" }))
        .mime(mime_type)
        .body(file.to_vec())
}
#[get("/state")]
fn get_state(req:Request) -> String {
    receive_state() 
}

#[post("/control/:")]
fn post_control(req: Request) -> Response {
    let wildcard = req.get_wildcard().unwrap();
    let params = UrlEncodedData::parse_str(req.get_parsed_body().unwrap_or(""));

    let code = match wildcard.as_str() {
        "run" => {
            send_commands(ControlCode::RUN);
            200
        }
        "stop" => {
            send_commands(ControlCode::STOP);
            200
        }
        "step" => {
            send_commands(ControlCode::STEP);
            200
        },
        "pokereg" => {
            send_commands(ControlCode::POKEREG { register: (Register::from_num(params.get_first("reg").unwrap_or("").parse().unwrap_or(0)).unwrap_or(Register::A0)), value: (params.get_first("val").unwrap_or("").parse().unwrap_or(0)) });
            200
        }
        _ => {
            500
        }
    };
    Response::new()
    .status_line(format!("HTTP/1.1 {} {}\r\n", code, match code { 500 => "Failed to set state", _ => "OK" })).body(receive_state().into())
}


#[post("/api/compile")]
fn post_compile(code: Option<&str>) -> Response {
    build_riscv(code, "RVGCC")
}
#[post("/api/assemble")]
fn post_assemble(code: Option<&str>) -> Response {
    build_riscv(code, "RVASM")
}
fn build_riscv(code: Option<&str>, lang: &str) -> Response {
    let (response_message, code, mime_type) = (|| { 
        /* first, check if the RVGCC environment variable is set - if not, we can't go any further */
        let rvgcc = match env::var(lang) {
            Ok(val) => val,
            Err(_) => { eprintln!("Compiler environment variable not set."); return (b"Compiler environment variable not set.".to_vec(),500, "text/plain"); }
        };
        /* if it is, great - now we check if there is any code to compile (passed as raw form data) */
        let args = match lang {
            "RVGCC" => &["-fpic", "-march=rv32i", "-mabi=ilp32", "-x", "c", "-", "-o", "/tmp/a.out"],
            _ => &["-o", "/tmp/a.out", "--", "","","","",""] // evil
        };
        if let Some(code) = code {
            /* if so, get cooking - spawn compiler process */
            let mut child = match Command::new(rvgcc)
                .args(args)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(_) => { eprintln!("Falied to launch compiler."); return (b"Falied to launch compiler\n".to_vec(),500,"text/plain"); }
            };
            /* pass the code to the compiler's stdin */
            if let Some(mut stdin) = child.stdin.take() {
                if let Err(_) = stdin.write_all(code.as_bytes()) { return (b"Falied to write to stdin of compiler\n".to_vec(),500,"text/plain"); }
            };
            /* now wait for the compiler to return */
            let output = match child.wait_with_output() {
                Ok(output) => output,
                Err(_) => { return (b"Compiler died somehow\n".to_vec(),500,"text/plain"); }
            };
            /* finally! now we can return the output of the compiler itself */
            if output.status.success() {
                return (fs::read("/tmp/a.out").unwrap(), 200, "application/octet-stream");
            } else {
                return (output.stderr, 500, "text/plain");
            }
        } else {
            (b"Nothing to compile\n".to_vec(), 500, "text/plain")
        }
    })();

    
    Response::new()
    .status_line(format!("HTTP/1.1 {} {}\r\n", code, match code { 500 => "Internal Server Error", _ => "OK" }))
        .mime(mime_type)
        .body(response_message)
}

// Helper functions to use the channels without this ungodly mess
fn send_commands(c: ControlCode) {
    let commands_tx = COMMANDS_TX.lock().unwrap().as_ref().unwrap().clone();
    commands_tx.send(c).unwrap();

}
fn receive_state() -> String {
    let mut state_rx_guard = STATE_RX.lock().unwrap(); // Get MutexGuard
    let state_rx = state_rx_guard.as_mut().unwrap();
    let s= state_rx.latest().clone();
    format!("{}", json!({
        "pc": s.pc,
        "cur_inst": s.cur_inst,
        "registers": s.registers,
        "cycle": s.cycle
    }))
}


pub fn run_server(commands_tx: CbSender<statetransfer::ControlCode>, state_rx: SvcReceiver<statetransfer::MachineState>) {
    *COMMANDS_TX.lock().unwrap() = Some(commands_tx);
    *STATE_RX.lock().unwrap() = Some(state_rx);

    let socket = TcpListener::bind(":::9001").unwrap();
    let routes = Routes::new(vec![get_assets(), post_compile(), post_assemble(),post_control(),get_state()]);
    let config = Config::new().routes(routes);
    let http = HttpListener::new(socket, config);

    println!("Web UI is listening on port 9001");

    http.start();
    println!("Started.");
}
