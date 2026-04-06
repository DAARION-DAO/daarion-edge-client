use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterfaceSurfaceType {
    NatsContract,
    HttpApi,
    SofiiaConsole,
    InstitutionalPortal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityInterfaceSurface {
    pub surface_id: String,
    pub surface_type: InterfaceSurfaceType,
    pub access_level: String,
    pub is_authenticated: bool,
}
