use serde::Deserialize;
use sqlx::{self, Database, Encode, encode::IsNull};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(transparent)]
#[schema(value_type = String, format = Email)]
pub struct EmailAddress(email_address::EmailAddress);

impl<'q, DB> Encode<'q, DB> for EmailAddress
where
    DB: Database,
    String: Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        self.0.to_string().encode_by_ref(buf)
    }
}

impl<DB> sqlx::Type<DB> for EmailAddress
where
    DB: Database,
    String: sqlx::Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as sqlx::Type<DB>>::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <String as sqlx::Type<DB>>::compatible(ty)
    }
}
