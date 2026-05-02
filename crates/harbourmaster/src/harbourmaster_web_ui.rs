use axum::{
    Router,
    body::Body,
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::get,
};
use mime_guess::from_path;
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(RustEmbed)]
#[folder = "build/client/"]
struct FrontendBuild;

pub struct HarbourmasterWebUi {
    build: FrontendBuild,
    listener: TcpListener,
    app: Router,
    address: SocketAddr,
}

impl HarbourmasterWebUi {
    pub async fn new() -> Self {
        let app: Router = Router::new().fallback(get(HarbourmasterWebUi::serve_ui));
        let addr: SocketAddr = SocketAddr::from(([0, 0, 0, 0], 3000));
        let listener: TcpListener = TcpListener::bind(addr).await.unwrap();

        Self {
            build: FrontendBuild,
            listener: listener,
            app: app,
            address: addr,
        }
    }

    async fn serve_ui(uri: Uri) -> impl IntoResponse {
        let mut path: String = uri.path().trim_start_matches('/').to_string();

        if path.is_empty() {
            path = "index.html".to_string();
        }

        match FrontendBuild::get(path.as_str()) {
            Some(content) => {
                let mime = from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(Body::from(content.data))
                    .unwrap()
            }
            None => match FrontendBuild::get("index.html") {
                Some(content) => Response::builder()
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from(content.data))
                    .unwrap(),
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from(
                        "Erreur : Fichier index.html introuvable dans le build. Impossible d'afficher l'interface armateur.",
                    ))
                    .unwrap(),
            },
        }
    }

    pub async fn start(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            axum::serve(self.listener, self.app).await.unwrap();
        })
    }
}
