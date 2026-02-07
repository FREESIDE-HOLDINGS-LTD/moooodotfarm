use crate::app::{AddCowHandler, ChangeCowCharacterHandler, GetHerdHandler};
use crate::config;
use crate::errors::{Error, Result};
use crate::{app, domain};
use anyhow::anyhow;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

pub mod generated {
    tonic::include_proto!("moooodotfarm.grpc");
}

use crate::domain::Character;
use generated::moooodotfarm_service_server::{MoooodotfarmService, MoooodotfarmServiceServer};
use generated::{
    AddCowRequest, AddCowResponse, ChangeCowCharacterRequest, ChangeCowCharacterResponse, Cow,
    GetHerdRequest, GetHerdResponse, Herd,
};

const DT_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

pub trait Deps {
    fn get_herd_handler(&self) -> &impl GetHerdHandler;
    fn add_cow_handler(&self) -> &impl AddCowHandler;
    fn change_cow_character_handler(&self) -> &impl ChangeCowCharacterHandler;
}

pub struct GrpcServer<'a, D> {
    config: &'a config::Config,
    deps: D,
}

impl<'a, D> GrpcServer<'a, D>
where
    D: Deps + Clone + Send + Sync + 'static,
{
    pub fn new(config: &'a config::Config, deps: D) -> Self {
        Self { config, deps }
    }

    pub async fn run(&self) -> Result<()> {
        let address = self
            .config
            .grpc_address()
            .parse::<std::net::SocketAddr>()
            .map_err(|err| Error::Unknown(anyhow!(err)))?;
        let service = HerdServiceImpl::new(self.deps.clone());

        Server::builder()
            .add_service(MoooodotfarmServiceServer::new(service))
            .serve(address)
            .await
            .map_err(|err| Error::Unknown(anyhow!(err)))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct HerdServiceImpl<D> {
    deps: D,
}

impl<D> HerdServiceImpl<D> {
    pub fn new(deps: D) -> Self {
        Self { deps }
    }
}

#[tonic::async_trait]
impl<D> MoooodotfarmService for HerdServiceImpl<D>
where
    D: Deps + Send + Sync + 'static,
{
    async fn get_herd(
        &self,
        _request: Request<GetHerdRequest>,
    ) -> std::result::Result<Response<GetHerdResponse>, Status> {
        let herd = self
            .deps
            .get_herd_handler()
            .get_herd()
            .map_err(|err| Status::internal(err.to_string()))?;
        let response = GetHerdResponse {
            herd: Some(Herd::from(&herd)),
        };

        Ok(Response::new(response))
    }

    async fn add_cow(
        &self,
        request: Request<AddCowRequest>,
    ) -> std::result::Result<Response<AddCowResponse>, Status> {
        let payload = request.into_inner();
        let name = domain::VisibleName::new(payload.name)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;
        let character = parse_character(&payload.character)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;
        let command = app::AddCow::new(name, character);

        self.deps
            .add_cow_handler()
            .add_cow(&command)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(AddCowResponse {}))
    }

    async fn change_cow_character(
        &self,
        request: Request<ChangeCowCharacterRequest>,
    ) -> std::result::Result<Response<ChangeCowCharacterResponse>, Status> {
        let payload = request.into_inner();
        let name = domain::VisibleName::new(payload.name)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;
        let character = parse_character(&payload.character)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;
        let command = app::ChangeCowCharacter::new(name, character);

        self.deps
            .change_cow_character_handler()
            .change_cow_character(&command)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(ChangeCowCharacterResponse {}))
    }
}

impl From<&app::Herd> for Herd {
    fn from(value: &app::Herd) -> Self {
        Self {
            cows: value.cows().iter().map(Cow::from).collect(),
        }
    }
}

impl From<&app::Cow> for Cow {
    fn from(value: &app::Cow) -> Self {
        let name_str = match value.name() {
            domain::Name::Visible(v) => v.url().to_string(),
            domain::Name::Censored(c) => c.url().to_string(),
        };
        let character_str = match value.character() {
            domain::Character::Brave => "brave",
            domain::Character::Shy => "shy",
        };
        let last_seen = value
            .last_seen()
            .map(|dt| dt.format(DT_FORMAT))
            .unwrap_or_default();
        let status_str = match value.status() {
            app::CowStatus::HappilyGrazing => "happily-grazing",
            app::CowStatus::RanAway => "ran-away",
            app::CowStatus::HaveNotCheckedYet => "have-not-checked-yet",
        };

        Self {
            name: name_str,
            character: character_str.to_string(),
            last_seen,
            status: status_str.to_string(),
        }
    }
}

fn parse_character(value: &str) -> Result<Character> {
    match value {
        "brave" => Ok(Character::Brave),
        "shy" => Ok(Character::Shy),
        other => Err(Error::Unknown(anyhow!("invalid character: {other}"))),
    }
}
