use actix_web::{test, App};
use pp_ss_r::app_config;
use std::io;
use std::sync::{Arc, Mutex, OnceLock};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone)]
struct SharedBuf(Arc<Mutex<Vec<u8>>>);

impl io::Write for SharedBuf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for SharedBuf {
    type Writer = SharedBuf;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

static BUF: OnceLock<Arc<Mutex<Vec<u8>>>> = OnceLock::new();

fn capture_buffer() -> Arc<Mutex<Vec<u8>>> {
    BUF.get_or_init(|| {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .json()
            .flatten_event(true)
            .without_time()
            .with_writer(SharedBuf(buf.clone()))
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
        buf
    })
    .clone()
}

#[actix_web::test]
async fn logs_request_json_with_timing_breakdown() {
    let buf = capture_buffer();
    buf.lock().unwrap().clear();

    let app = test::init_service(App::new().configure(app_config)).await;
    // /pay renders locally (no fetch) and logs the load_ms/render_ms breakdown.
    let req = test::TestRequest::post().uri("/pay").to_request();
    let _ = test::call_service(&app, req).await;

    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    let line = text
        .lines()
        .find(|l| l.contains("\"message\":\"request\""))
        .expect("a JSON request log line must be emitted");

    let v: serde_json::Value = serde_json::from_str(line).expect("log line must be valid JSON");
    assert_eq!(v["message"].as_str(), Some("request"));
    assert_eq!(v["method"].as_str(), Some("POST"));
    assert_eq!(v["path"].as_str(), Some("/pay"));
    assert_eq!(v["status"].as_u64(), Some(200));
    assert!(v["load_ms"].is_number(), "load_ms present and numeric");
    assert!(v["render_ms"].is_number(), "render_ms present and numeric");
    assert!(v["total_ms"].is_number(), "total_ms present and numeric");
    assert!(v["ts_ms"].is_number(), "ts_ms present and numeric");
    assert!(v["bytes"].as_u64().unwrap() > 0, "bytes reflects body length");
    assert!(
        v["total_ms"].as_f64().unwrap() >= v["render_ms"].as_f64().unwrap(),
        "total time covers render time"
    );
}
