use std::collections::BTreeSet;

use adoc_application::search::{
    HybridSearchIndex, ScopedSearchRequest, SearchDrift, SearchHit, SearchIndexResult,
    SearchPermissionKey, SearchProjection, SearchRetrievalError, SearchSourceKind, projection_id,
};
use adoc_ports::BoxFuture;
use reqwest::{Client, StatusCode, Url};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

const PERMISSION_BATCH: usize = 4_096;
const MSEARCH_BATCH: usize = 16;

#[derive(Clone)]
pub struct OpenSearchRetrievalIndex {
    client: Client,
    endpoint: Url,
    prefix: String,
    credential: Option<Credential>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Credential {
    username: String,
    password: String,
}

#[derive(Clone)]
enum QueryPlan {
    Lexical {
        kind: SearchSourceKind,
        keys: BTreeSet<String>,
    },
    Vector {
        kind: SearchSourceKind,
        keys: BTreeSet<String>,
    },
    Watermark,
    Drift,
}

struct PlannedQuery {
    alias: String,
    plan: QueryPlan,
    body: Value,
}

impl OpenSearchRetrievalIndex {
    pub fn new(
        endpoint: Url,
        prefix: String,
        credential_json: Option<&str>,
    ) -> Result<Self, SearchRetrievalError> {
        if prefix.is_empty() {
            return Err(SearchRetrievalError::Validation);
        }
        let credential = credential_json
            .map(serde_json::from_str)
            .transpose()
            .map_err(|_| SearchRetrievalError::Validation)?;
        let client = Client::builder()
            .build()
            .map_err(|_| SearchRetrievalError::Internal)?;
        Ok(Self {
            client,
            endpoint,
            prefix,
            credential,
        })
    }

    async fn execute(
        &self,
        request: &ScopedSearchRequest,
    ) -> Result<SearchIndexResult, SearchRetrievalError> {
        let mut queries = Vec::new();
        self.plan_kind(
            &mut queries,
            SearchSourceKind::Published,
            &request.published_keys,
            request,
        );
        self.plan_kind(
            &mut queries,
            SearchSourceKind::Draft,
            &request.draft_keys,
            request,
        );
        let generation = self
            .generation(
                !request.published_keys.is_empty(),
                !request.draft_keys.is_empty(),
            )
            .await?;
        let mut lexical_hits = Vec::new();
        let mut vector_hits = Vec::new();
        let mut drift = BTreeSet::new();
        let mut watermark = 0_i64;
        for batch in queries.chunks(MSEARCH_BATCH) {
            let response = self.msearch(request.workspace_id, batch).await?;
            let responses = response
                .get("responses")
                .and_then(Value::as_array)
                .ok_or(SearchRetrievalError::Unavailable)?;
            if responses.len() != batch.len() {
                return Err(SearchRetrievalError::Unavailable);
            }
            for (query, response) in batch.iter().zip(responses) {
                if response.get("error").is_some() {
                    return Err(SearchRetrievalError::Unavailable);
                }
                match &query.plan {
                    QueryPlan::Lexical { kind, keys } => lexical_hits.extend(parse_hits(
                        response,
                        request.workspace_id,
                        *kind,
                        keys,
                    )?),
                    QueryPlan::Vector { kind, keys } => {
                        vector_hits.extend(parse_hits(response, request.workspace_id, *kind, keys)?)
                    }
                    QueryPlan::Watermark => {
                        watermark = watermark.max(
                            response
                                .pointer("/aggregations/watermark/value")
                                .and_then(Value::as_f64)
                                .map(|value| value as i64)
                                .unwrap_or(0),
                        );
                    }
                    QueryPlan::Drift => {
                        for hit in hit_values(response)? {
                            let source = hit
                                .get("_source")
                                .ok_or(SearchRetrievalError::Unavailable)?;
                            let document_id = source
                                .get("document_id")
                                .and_then(Value::as_str)
                                .and_then(|value| Uuid::parse_str(value).ok())
                                .ok_or(SearchRetrievalError::Unavailable)?;
                            let fingerprint = source
                                .get("permission_fingerprint")
                                .and_then(Value::as_str)
                                .filter(|value| value.len() == 64)
                                .ok_or(SearchRetrievalError::Unavailable)?;
                            drift.insert((document_id, fingerprint.to_owned()));
                        }
                    }
                }
            }
        }
        Ok(SearchIndexResult {
            lexical_hits,
            vector_hits,
            drift: drift
                .into_iter()
                .map(|(document_id, detected_fingerprint)| SearchDrift {
                    document_id,
                    detected_fingerprint,
                })
                .collect(),
            index_generation: generation,
            index_watermark: watermark,
        })
    }

    fn plan_kind(
        &self,
        queries: &mut Vec<PlannedQuery>,
        kind: SearchSourceKind,
        keys: &[SearchPermissionKey],
        request: &ScopedSearchRequest,
    ) {
        let alias = self.read_alias(kind);
        for batch in keys.chunks(PERMISSION_BATCH) {
            let composite = batch
                .iter()
                .map(|key| key.composite_key.clone())
                .collect::<BTreeSet<_>>();
            let scopes = batch
                .iter()
                .map(|key| key.scope_token.clone())
                .collect::<BTreeSet<_>>();
            let filter = safe_filter(request.workspace_id, &composite);
            queries.push(PlannedQuery {
                alias: alias.clone(),
                plan: QueryPlan::Lexical {
                    kind,
                    keys: composite.clone(),
                },
                body: json!({
                    "size":100,
                    "track_total_hits":false,
                    "_source":source_fields(),
                    "query":{"bool":{"filter":filter,"must":[{"multi_match":{
                        "query":request.normalized_query,"fields":["title^3","body"]
                    }}]}},
                    "sort":[{"_score":"desc"},{"_id":"asc"}]
                }),
            });
            if let Some(vector) = &request.query_vector {
                queries.push(PlannedQuery {
                    alias: alias.clone(),
                    plan: QueryPlan::Vector {
                        kind,
                        keys: composite.clone(),
                    },
                    body: json!({
                        "size":100,
                        "track_total_hits":false,
                        "_source":source_fields(),
                        "query":{"knn":{"embedding":{
                            "vector":vector,"k":100,"filter":{"bool":{"filter":filter}}
                        }}},
                        "sort":[{"_score":"desc"},{"_id":"asc"}]
                    }),
                });
            }
            queries.push(PlannedQuery {
                alias: alias.clone(),
                plan: QueryPlan::Watermark,
                body: json!({
                    "size":0,"track_total_hits":false,
                    "query":{"bool":{"filter":safe_filter(request.workspace_id,&composite)}},
                    "aggs":{"watermark":{"max":{"field":"outbox_sequence"}}}
                }),
            });
            queries.push(PlannedQuery {
                alias: alias.clone(),
                plan: QueryPlan::Drift,
                body: json!({
                    "size":100,"track_total_hits":false,
                    "_source":["document_id","permission_fingerprint"],
                    "query":{"bool":{
                        "filter":[
                            {"term":{"workspace_id":request.workspace_id}},
                            {"term":{"deleted":false}},
                            {"terms":{"permission_scope":scopes}}
                        ],
                        "must_not":[{"terms":{"permission_key":composite}}]
                    }},
                    "sort":[{"_id":"asc"}]
                }),
            });
        }
    }

    async fn msearch(
        &self,
        workspace_id: Uuid,
        queries: &[PlannedQuery],
    ) -> Result<Value, SearchRetrievalError> {
        let mut body = String::new();
        for query in queries {
            body.push_str(
                &serde_json::to_string(&json!({
                    "index":query.alias,"routing":workspace_id
                }))
                .map_err(|_| SearchRetrievalError::Internal)?,
            );
            body.push('\n');
            body.push_str(
                &serde_json::to_string(&query.body).map_err(|_| SearchRetrievalError::Internal)?,
            );
            body.push('\n');
        }
        let response = self
            .send(
                self.client
                    .post(self.url("_msearch")?)
                    .header("content-type", "application/x-ndjson")
                    .body(body),
            )
            .await?;
        if !response.status().is_success() {
            return Err(search_status(response.status()));
        }
        response
            .json()
            .await
            .map_err(|_| SearchRetrievalError::Unavailable)
    }

    async fn generation(
        &self,
        published: bool,
        draft: bool,
    ) -> Result<String, SearchRetrievalError> {
        let aliases = [
            published.then(|| self.read_alias(SearchSourceKind::Published)),
            draft.then(|| self.read_alias(SearchSourceKind::Draft)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        let response = self
            .send(
                self.client
                    .get(self.url(&format!("_alias/{}", aliases.join(",")))?),
            )
            .await?;
        if !response.status().is_success() {
            return Err(search_status(response.status()));
        }
        let value: Value = response
            .json()
            .await
            .map_err(|_| SearchRetrievalError::Unavailable)?;
        let mut indices = value
            .as_object()
            .ok_or(SearchRetrievalError::Unavailable)?
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        indices.sort();
        Ok(hex::encode(Sha256::digest(indices.join("\0").as_bytes())))
    }

    async fn send(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, SearchRetrievalError> {
        let request = if let Some(credential) = &self.credential {
            request.basic_auth(&credential.username, Some(&credential.password))
        } else {
            request
        };
        request
            .send()
            .await
            .map_err(|_| SearchRetrievalError::Unavailable)
    }

    fn read_alias(&self, kind: SearchSourceKind) -> String {
        format!("{}-{}-read", self.prefix, kind_name(kind))
    }

    fn url(&self, path: &str) -> Result<Url, SearchRetrievalError> {
        self.endpoint
            .join(path)
            .map_err(|_| SearchRetrievalError::Internal)
    }
}

impl HybridSearchIndex for OpenSearchRetrievalIndex {
    fn retrieve<'a>(
        &'a self,
        request: &'a ScopedSearchRequest,
    ) -> BoxFuture<'a, Result<SearchIndexResult, SearchRetrievalError>> {
        Box::pin(async move { self.execute(request).await })
    }
}

fn safe_filter(workspace_id: Uuid, keys: &BTreeSet<String>) -> Vec<Value> {
    vec![
        json!({"term":{"workspace_id":workspace_id}}),
        json!({"term":{"deleted":false}}),
        json!({"terms":{"permission_key":keys}}),
    ]
}

fn source_fields() -> Value {
    Value::Bool(true)
}

fn parse_hits(
    response: &Value,
    workspace_id: Uuid,
    kind: SearchSourceKind,
    keys: &BTreeSet<String>,
) -> Result<Vec<SearchHit>, SearchRetrievalError> {
    hit_values(response)?
        .iter()
        .map(|value| {
            let stable_id = value
                .get("_id")
                .and_then(Value::as_str)
                .ok_or(SearchRetrievalError::Unavailable)?
                .to_owned();
            let provider_score = value
                .get("_score")
                .and_then(Value::as_f64)
                .ok_or(SearchRetrievalError::Unavailable)?;
            let projection: SearchProjection = serde_json::from_value(
                value
                    .get("_source")
                    .cloned()
                    .ok_or(SearchRetrievalError::Unavailable)?,
            )
            .map_err(|_| SearchRetrievalError::Unavailable)?;
            if projection.workspace_id != workspace_id
                || projection.source_kind != kind.as_str()
                || projection.deleted
                || !keys.contains(&projection.permission_key)
                || stable_id
                    != projection_id(
                        workspace_id,
                        kind,
                        projection.document_id,
                        projection.region_id,
                    )
            {
                return Err(SearchRetrievalError::Unavailable);
            }
            Ok(SearchHit {
                stable_id,
                source_kind: kind,
                document_id: projection.document_id,
                source_revision: projection.source_revision,
                version_number: projection.version_number,
                region_id: projection.region_id,
                title: projection.title,
                body: projection.body,
                terms: projection.terms,
                snapshot_hash: projection.snapshot_hash,
                updated_at: projection.updated_at,
                outbox_sequence: projection.outbox_sequence,
                provider_score,
            })
        })
        .collect()
}

fn hit_values(response: &Value) -> Result<&Vec<Value>, SearchRetrievalError> {
    response
        .pointer("/hits/hits")
        .and_then(Value::as_array)
        .ok_or(SearchRetrievalError::Unavailable)
}

fn search_status(status: StatusCode) -> SearchRetrievalError {
    if status == StatusCode::TOO_MANY_REQUESTS
        || status == StatusCode::NOT_FOUND
        || status.is_server_error()
    {
        SearchRetrievalError::Unavailable
    } else {
        SearchRetrievalError::Internal
    }
}

fn kind_name(kind: SearchSourceKind) -> &'static str {
    match kind {
        SearchSourceKind::Published => "published",
        SearchSourceKind::Draft => "draft",
    }
}
