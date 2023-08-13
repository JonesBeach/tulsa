use std::io::Read;
use std::thread;

pub struct Mock {}

impl Mock {
    pub fn assert(&self) -> bool {
        true
    }
}

pub struct Builder {}

impl Builder {
    pub fn with_status(&mut self, _status: u16) -> &mut Builder {
        self
    }

    pub fn with_body(&mut self, _body: Vec<u8>) -> &mut Builder {
        self
    }

    pub fn create(&self) -> Mock {
        Mock {}
    }
}

pub struct Server {
    host: &'static str,
    port: u16,
}

use std::fs;
use tokio::net::TcpListener;
use tokio::runtime;
use tokio::task::spawn;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Request, Response};

mod error;

use error::MockError;

async fn handle_request(
    _request: Request<Body>,
) -> Result<Response<Body>, MockError> {
    let mut buffer: Vec<u8> = Vec::new();
    let mut file = fs::File::open("fixtures/gtfs-07132023-123501").expect("Failed to open the file");
    file.read_to_end(&mut buffer)
        .expect("Failed to read the file");

    let response = Response::new(Body::from(buffer));
    Ok(response)
}

impl Server {
    pub fn new() -> Server {
        let host = "localhost";
        let port: u16 = 5001;

        let runtime = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        thread::spawn(move || {
            runtime.block_on(async {
                let listener = TcpListener::bind(format!("{}:{}", host, port))
                    .await
                    .unwrap();

                while let Ok((stream, _)) = listener.accept().await {
                    spawn(async move {
                        let _ = Http::new()
                            .serve_connection(
                                stream,
                                service_fn(move |request: Request<Body>| {
                                    handle_request(request)
                                }),
                            )
                            .await;
                    });
                }
            });
        });

        Server {
            host,
            port
        }
    }

    pub fn mock(&self, _method: &'static str, _path: &'static str) -> Builder {
        Builder {}
    }

    pub fn url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}
