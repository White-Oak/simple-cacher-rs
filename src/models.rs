use super::schema::responses;

#[derive(Queryable)]
pub struct CachedResponseDB {
    pub id: i32,
    pub status: i16,
    pub request: String,
    pub body: String
}

#[derive(Serialize, Deserialize, Debug, Insertable, Clone)]
#[table_name="responses"]
pub struct CachedResponse{
    pub status: i16,
    pub request: String,
    pub body: String,
}

impl From<CachedResponseDB> for CachedResponse {
    fn from(s: CachedResponseDB) -> Self {
        CachedResponse {status: s.status, request: s.request, body: s.body}
    }
}
