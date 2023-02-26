use std::fmt::Formatter;
use std::net::SocketAddr;
use std::str::FromStr;

use core::fmt::Debug;

use chatgpt_proxy_server::*;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Request, Response, Server, Uri};
use hyper_tls::HttpsConnector;
use lazy_static::lazy_static;

use dotenv;
use tokio::sync::Mutex;

// create config struct
struct Config {
    chatgpt_url: String,
    ratelimit: u32,
}

lazy_static! {
    static ref CONFIG: Config = {
        dotenv::dotenv().ok();
        let chatgpt_url = std::env::var("CHATGPT_URL").expect("CHATGPT_URL is not set");
        // Rate limit for 1/min
        let ratelimit = std::env::var("RATELIMIT")
            .unwrap_or("100".to_string())
            .parse::<u32>()
            .unwrap();
        Config {
            chatgpt_url,
            ratelimit,
        }
    };
}
// Add Debug log for CONFIG
impl Debug for CONFIG {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("chatgpt_url", &self.chatgpt_url)
            .field("ratelimit", &self.ratelimit)
            .finish()
    }
}

lazy_static! {
    static ref RATE_LIMITER: Mutex<RateLimiter> = Mutex::new(RateLimiter::new(CONFIG.ratelimit));
}

async fn proxy(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let client_ip = &req
        .extensions()
        .get::<SocketAddr>()
        .unwrap()
        .ip()
        .to_string();
    // Read Client IP read form header or client
    let real_ip = req
        .headers()
        .get("X-Forwarded-For")
        .map(|x| x.to_str().unwrap())
        .unwrap_or_else(|| &client_ip);
    // Check rate limit
    let mut rate_limiter = RATE_LIMITER.lock().await;
    if !rate_limiter.check_rate_limit(real_ip) {
        return Ok(Response::builder()
            .status(429)
            .header("Content-Type", "text/plain")
            .body(Body::from("Too Many Requests"))
            .unwrap());
    }
    // try forward request
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    // If path switch with /backend-api, change it to /api
    let path_and_query = req.uri().path_and_query().unwrap().to_string();
    let path_and_query = if path_and_query.starts_with("/backend-api") {
        path_and_query.replace("/backend-api", "/api")
    } else {
        path_and_query.to_string()
    };
    // Create new uri
    let uri = Uri::from_str(&format!("{}{}", CONFIG.chatgpt_url, path_and_query,)).unwrap();
    println!("uri: {:?}", uri);
    let request_builder = Request::builder().method(req.method())
        .uri(uri)
        .header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36 Edg/110.0.1587.50")
        .header("Authorization", req.headers().get("Authorization").unwrap().to_str().unwrap_or_else(|_| "null"))
        .header("Content-Type", req.headers().get("Content-Type").unwrap().to_str().unwrap_or_else(|_| "application/json"));

    let response = client
        .request(request_builder.body(req.into_body()).unwrap())
        .await
        .unwrap();

    Ok(Response::builder()
        .status(response.status())
        .header(
            "Content-Type",
            response
                .headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .body(response.into_body())
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port = std::env::var("PORT")
        .unwrap_or("3000".to_string())
        .parse::<u16>()
        .unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let service = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let mut req = req;
                req.extensions_mut().insert(remote_addr);
                proxy(req)
            }))
        }
    });
    let server = Server::bind(&addr).serve(service);
    // print config
    println!("Config: {:?}", CONFIG);
    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
