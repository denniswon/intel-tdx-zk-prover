use chrono::{DateTime, Utc};
use ethereum_types::Address;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::encode::IsNull;

use sqlx::error::BoxDynError;
use sqlx::{Decode, Encode, Type};

use ethereum_types::H160;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvmAddress(pub H160);

impl std::fmt::Display for EvmAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl From<H160> for EvmAddress {
    fn from(addr: H160) -> Self {
        Self(addr)
    }
}

impl From<EvmAddress> for H160 {
    fn from(evm_addr: EvmAddress) -> Self {
        evm_addr.0
    }
}

impl<'r, DB> Decode<'r, DB> for EvmAddress
where
    DB: sqlx::Database,
    &'r str: Decode<'r, DB>,
{
    fn decode(value: <DB as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <&str as Decode<DB>>::decode(value)?;
        Ok(EvmAddress(H160::from_str(s)?))
    }
}

impl<'a, DB> Encode<'a, DB> for EvmAddress
where
    DB: sqlx::Database,
    String: Encode<'a, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<IsNull, BoxDynError> {
        <String as Encode<DB>>::encode_by_ref(&format!("{:#x}", self.0), buf)
    }

    fn size_hint(&self) -> usize {
        <String as Encode<DB>>::size_hint(&format!("{:#x}", self.0))
    }
}

impl<DB> Type<DB> for EvmAddress
where
    DB: sqlx::Database,
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as Type<DB>>::compatible(ty)
    }
}

#[derive(Clone, Deserialize, Serialize, FromRow)]
pub struct Agent {
    pub id: i32,
    pub agent_name: String,
    pub agent_type: String,
    pub agent_uri: String,
    pub agent_description: String,
    pub agent_owner: Address,
    pub agent_status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(strum_macros::Display, Debug, Clone, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "agent_status")]
pub enum AgentStatus {
    Active,
    Inactive,
}
