use crate::ping::ping_service_server::{PingService, PingServiceServer};
use clap::Parser;
use clap_derive::Parser;
use tonic::transport::{Identity, Server, ServerTlsConfig};

mod ping {
    tonic::include_proto!("ping");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("ping_descriptor");
}

pub(crate) mod cli {
    use clap_derive::Subcommand;

    #[derive(Subcommand)]
    pub(crate) enum Commands {
        #[clap(name = "serve", about = "Start serving the GRPC service")]
        Serve {
            #[clap(
                long("listen"),
                short('l'),
                num_args(1),
                help("Address and port to listen on"),
                default_value = "0.0.0.0:8080"
            )]
            listen: String,
            #[clap(long("tls-key"), num_args(1), help("Location of the TLS private key"))]
            tls_key: Option<std::path::PathBuf>,
            #[clap(long("tls-cert"), num_args(1), help("Location of the TLS certificate"))]
            tls_cert: Option<std::path::PathBuf>,
        },
    }
}

#[derive(Parser)]
#[clap(
    author = "andlaz <andras.szerdahelyi@gmail.com>",
    version,
    about = "GRPC ingress reachability canary"
)]
struct Cli {
    #[clap(subcommand)]
    pub command: cli::Commands,
}

#[derive(Default)]
struct Ping {}

#[tonic::async_trait]
impl PingService for Ping {
    async fn ping(
        &self,
        _: tonic::Request<ping::PingMessage>,
    ) -> Result<tonic::Response<ping::Pong>, tonic::Status> {
        let response = ping::Pong {};
        Ok(tonic::Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match &cli.command {
        cli::Commands::Serve {
            listen,
            tls_key,
            tls_cert,
        } => {
            let addr = listen
                .parse()
                .map_err(|e| format!("Failed to parse listen address: {:?}", e))?;

            let reflection = tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(ping::FILE_DESCRIPTOR_SET)
                .build_v1()
                .unwrap();

            match (tls_key, tls_cert) {
                // TLS
                (Some(tls_key), Some(tls_cert)) => {
                    rustls::crypto::aws_lc_rs::default_provider()
                        .install_default()
                        .map_err(|e| {
                            format!("Failed to install aws_lc_rs as default provider: {:?}", e)
                        })?;

                    let tls_key = std::fs::read_to_string(tls_key)
                        .map_err(|e| format!("Failed to read TLS key: {:?}", e))?;
                    let tls_cert = std::fs::read_to_string(tls_cert)
                        .map_err(|e| format!("Failed to read TLS cert: {:?}", e))?;

                    Server::builder()
                        .tls_config(
                            ServerTlsConfig::new().identity(Identity::from_pem(tls_cert, tls_key)),
                        )
                        .map_err(|e| format!("Failed to create TLS config: {:?}", e))?
                        .add_service(reflection)
                        .add_service(PingServiceServer::new(Ping {}))
                        .serve(addr)
                        .await
                        .map_err(|e| format!("Failed to serve: {:?}", e))?;

                    Ok(())
                }
                // non-TLS
                (None, None) => {
                    Server::builder()
                        .add_service(reflection)
                        .add_service(PingServiceServer::new(Ping {}))
                        .serve(addr)
                        .await
                        .map_err(|e| format!("Failed to serve: {:?}", e))?;

                    Ok(())
                }
                _ => Err("TLS cert & key must either both be provided or omitted".to_string()),
            }
        }
    }
}
