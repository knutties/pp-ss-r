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
async fn api_request_is_logged_with_timing() {
    let buf = capture_buffer();
    buf.lock().unwrap().clear();

    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get()
        .uri("/api/orders/log_me?delay_ms=30")
        .to_request();
    let _ = test::call_service(&app, req).await;

    let text = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    let line = text
        .lines()
        .find(|l| l.contains("\"message\":\"request\"") && l.contains("/api/orders/log_me"))
        .expect("an /api request log line must be emitted");

    let v: serde_json::Value = serde_json::from_str(line).expect("log line must be valid JSON");
    assert_eq!(v["method"].as_str(), Some("GET"));
    assert_eq!(v["path"].as_str(), Some("/api/orders/log_me"));
    assert_eq!(v["status"].as_u64(), Some(200));
    assert_eq!(v["delay_ms"].as_u64(), Some(30), "requested delay is recorded");
    assert!(v["serialize_ms"].is_number(), "serialize_ms present");
    assert!(v["total_ms"].is_number(), "total_ms present");
    assert!(v["bytes"].as_u64().unwrap() > 0, "bytes reflects body length");
    assert!(
        v["total_ms"].as_f64().unwrap() >= 25.0,
        "total_ms includes the simulated delay"
    );
}
