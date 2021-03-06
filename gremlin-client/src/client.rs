use crate::io::GraphSON;
use crate::message::{gremlin, Message, Response};
use crate::pool::GremlinConnectionManager;
use crate::ToGValue;
use crate::{ConnectionOptions, GremlinError, GremlinResult};
use crate::{GResultSet, GValue};
use r2d2::Pool;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Debug)]
pub struct GremlinClient {
    pool: Pool<GremlinConnectionManager>,
    io: GraphSON,
    alias: Option<String>,
}

impl GremlinClient {
    pub fn connect<T>(options: T) -> GremlinResult<GremlinClient>
    where
        T: Into<ConnectionOptions>,
    {
        let opts = options.into();
        let pool_size = opts.pool_size;
        let manager = GremlinConnectionManager::new(opts);

        let pool = Pool::builder().max_size(pool_size).build(manager)?;

        Ok(GremlinClient {
            pool,
            io: GraphSON::V3,
            alias: None,
        })
    }

    /// Return a cloned client with the provided alias
    pub fn alias<T>(&self, alias: T) -> GremlinClient
    where
        T: Into<String>,
    {
        let mut cloned = self.clone();
        cloned.alias = Some(alias.into());
        cloned
    }

    pub fn execute<T>(
        &self,
        script: T,
        params: &[(&str, &dyn ToGValue)],
    ) -> GremlinResult<GResultSet>
    where
        T: Into<String>,
    {
        let map: HashMap<String, GValue> = params
            .iter()
            .map(|(k, v)| (String::from(*k), v.to_gvalue()))
            .collect();

        let p = self.io.write(&GValue::Map(map))?;

        let message = gremlin(
            script.into(),
            p.as_object()
                .ok_or_else(|| GremlinError::Json(String::from("it should be an object")))?
                .clone(),
            self.alias.clone(),
        );

        self.send_message(message)
    }

    fn send_message<T: Serialize>(&self, msg: Message<T>) -> GremlinResult<GResultSet> {
        let message = self.build_message(msg)?;

        let mut conn = self.pool.get()?;

        let content_type = "application/json";
        let payload = String::from("") + content_type + &message;
        let mut binary = payload.into_bytes();
        binary.insert(0, content_type.len() as u8);

        conn.send(binary)?;

        let (response, results) = self.read_response(&mut conn)?;

        Ok(GResultSet::new(self.clone(), results, response, conn))
    }

    pub(crate) fn read_response(
        &self,
        conn: &mut r2d2::PooledConnection<GremlinConnectionManager>,
    ) -> GremlinResult<(Response, VecDeque<GValue>)> {
        let result = conn.recv()?;

        let response: Response = serde_json::from_slice(&result)?;

        match response.status.code {
            200 | 206 => {
                let results: VecDeque<GValue> = self
                    .io
                    .read(&response.result.data)?
                    .map(|v| v.into())
                    .unwrap_or_else(VecDeque::new);

                Ok((response, results))
            }
            204 => Ok((response, VecDeque::new())),
            _ => Err(GremlinError::Request((
                response.status.code,
                response.status.message,
            ))),
        }
    }
    fn build_message<T: Serialize>(&self, msg: Message<T>) -> GremlinResult<String> {
        serde_json::to_string(&msg).map_err(GremlinError::from)
    }
}
