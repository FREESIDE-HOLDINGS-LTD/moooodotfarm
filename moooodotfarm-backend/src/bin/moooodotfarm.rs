use clap::{Command, arg};
use env_logger::Env;
use log::error;
use moooodotfarm_backend::adapters::{ConfigLoader, database};
use moooodotfarm_backend::app::get_herd::GetHerdHandler;
use moooodotfarm_backend::app::update::UpdateHandler;
use moooodotfarm_backend::config::Config;
use moooodotfarm_backend::errors::Result;
use moooodotfarm_backend::ports::http;
use moooodotfarm_backend::ports::timers;
use moooodotfarm_backend::{adapters, app, domain};
use prometheus::Registry;

fn cli() -> Command {
    Command::new("moooodotfarm")
        .about("Software which herds cows.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .about("Runs the program")
                .arg(arg!(<CONFIG> "Path to the configuration file")),
        )
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().filter_or("RUST_LOG", "info")).init();

    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("run", sub_matches)) => {
            let config_file_path = sub_matches.try_get_one::<String>("CONFIG")?.unwrap();
            run(config_file_path).await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

async fn run(config_file_path: &str) -> Result<()> {
    let config_loader = ConfigLoader::new(config_file_path);
    let config = config_loader.load()?;

    let metrics = adapters::Metrics::new()?;

    let database = database::Database::new(config.database_path())?;
    let downloader = adapters::CowTxtDownloader::new();

    let cows = config.cows().to_vec();
    let herd = domain::Herd::new(cows)?;

    let update_handler = UpdateHandler::new(
        herd.clone(),
        database.clone(),
        downloader.clone(),
        metrics.clone(),
    );
    let get_herd_handler = GetHerdHandler::new(herd.clone(), database.clone(), metrics.clone());

    let mut timer = timers::UpdateTimer::new(update_handler);
    let server = http::Server::new();

    tokio::spawn({
        async move {
            timer.run().await;
        }
    });

    let http_deps = HttpDeps::new(get_herd_handler, metrics);

    server_loop(&server, &config, http_deps).await;
    Ok(())
}

async fn server_loop<D>(server: &http::Server, config: &Config, deps: D)
where
    D: http::Deps + Sync + Send + Clone + 'static,
{
    loop {
        match server.run(config, deps.clone()).await {
            Ok(_) => {
                error!("the server exited without returning any errors")
            }
            Err(err) => {
                error!("the server exited with an error: {err}")
            }
        }
    }
}

#[derive(Clone)]
struct HttpDeps<GHH> {
    get_herd_handler: GHH,
    metrics: adapters::Metrics,
}

impl<GHH> HttpDeps<GHH> {
    pub fn new(get_herd_handler: GHH, metrics: adapters::Metrics) -> Self {
        Self {
            get_herd_handler,
            metrics,
        }
    }
}

impl<GHH> http::Deps for HttpDeps<GHH>
where
    GHH: app::GetHerdHandler,
{
    fn get_herd_handler(&self) -> &impl app::GetHerdHandler {
        &self.get_herd_handler
    }

    fn metrics(&self) -> &Registry {
        self.metrics.registry()
    }
}
