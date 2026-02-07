use clap::{Command, arg};
use env_logger::Env;
use log::error;
use moooodotfarm_backend::adapters::{ConfigLoader, database};
use moooodotfarm_backend::app::CowTxtDownloader;
use moooodotfarm_backend::app::get_herd::GetHerdHandler;
use moooodotfarm_backend::app::update::UpdateHandler;
use moooodotfarm_backend::config::Config;
use moooodotfarm_backend::domain::VisibleName;
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
        .subcommand(
            Command::new("check")
                .about("Checks up on a cow")
                .arg(arg!(<URL> "URL of the cow")),
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
        Some(("check", sub_matches)) => {
            let url = sub_matches.try_get_one::<String>("URL")?.unwrap();
            check(url).await?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

async fn run(config_file_path: &str) -> Result<()> {
    let config = &ConfigLoader::new(config_file_path).load()?;
    let service = Service::new(config)?;

    tokio::spawn({
        async move {
            service.update_timer.run().await;
        }
    });

    server_loop(&service.http_server).await;
    Ok(())
}

async fn check(url: &str) -> Result<()> {
    let downloader = adapters::CowTxtDownloader::new();
    let name = VisibleName::new(url)?;
    let cow_txt = downloader.download(&name).await?;
    println!("{}", cow_txt);
    println!("Cow is ok!");
    Ok(())
}

async fn server_loop<'a, D>(server: &http::Server<'a, D>)
where
    D: http::Deps + Sync + Send + Clone + 'static,
{
    loop {
        match server.run().await {
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

type GetHerdHandlerImpl = GetHerdHandler<database::Database, adapters::Metrics>;
type UpdateHandlerImpl =
    UpdateHandler<database::Database, adapters::CowTxtDownloader, adapters::Metrics>;
type HttpDepsImpl = HttpDeps<GetHerdHandlerImpl>;
type HttpServerImpl<'a> = http::Server<'a, HttpDepsImpl>;
type UpdateTimerImpl = timers::UpdateTimer<UpdateHandlerImpl>;

struct Service<'a> {
    _get_herd_handler: GetHerdHandlerImpl,
    update_handler: UpdateHandlerImpl,

    http_server: HttpServerImpl<'a>,
    update_timer: UpdateTimerImpl,
}

impl<'a> Service<'a> {
    fn new(config: &'a Config) -> Result<Self> {
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

        let timer = timers::UpdateTimer::new(update_handler.clone());
        let http_deps = HttpDeps::new(get_herd_handler.clone(), metrics);
        let server = http::Server::new(config, http_deps);

        Ok(Self {
            _get_herd_handler: get_herd_handler,
            update_handler,
            http_server: server,
            update_timer: timer,
        })
    }
}
