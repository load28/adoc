use adoc_application::search::{
    ProjectionMutation, SEARCH_PROJECTION_SCHEMA, SearchIndex, SearchProjection,
    SearchProjectionError, SearchSourceKind, TOMBSTONE_REGION_ID, permission_scope_token,
    projection_id,
};
use adoc_ports::BoxFuture;
use chrono::Utc;
use reqwest::{Client, StatusCode, Url};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Clone)]
pub struct OpenSearchIndex {
    client: Client,
    endpoint: Url,
    prefix: String,
    embedding_dimension: u32,
    credential: Option<Credential>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Credential {
    username: String,
    password: String,
}

impl OpenSearchIndex {
    pub fn new(
        endpoint: Url,
        prefix: String,
        embedding_dimension: u32,
        credential_json: Option<&str>,
    ) -> Result<Self, SearchProjectionError> {
        if prefix.is_empty() || embedding_dimension == 0 {
            return Err(SearchProjectionError::Permanent("SEARCH_CONFIG_INVALID"));
        }
        let credential = credential_json
            .map(serde_json::from_str)
            .transpose()
            .map_err(|_| SearchProjectionError::Permanent("SEARCH_CREDENTIAL_INVALID"))?;
        Ok(Self {
            client: Client::builder()
                .build()
                .map_err(|_| SearchProjectionError::Permanent("SEARCH_CLIENT_INVALID"))?,
            endpoint,
            prefix,
            embedding_dimension,
            credential,
        })
    }

    pub async fn bootstrap(&self) -> Result<(), SearchProjectionError> {
        for kind in [SearchSourceKind::Published, SearchSourceKind::Draft] {
            self.bootstrap_kind(kind).await?;
        }
        Ok(())
    }

    pub async fn rebuild(
        &self,
        generation: i64,
        mutations: &[ProjectionMutation],
    ) -> Result<(), SearchProjectionError> {
        self.prepare_rebuild(generation).await?;
        self.activate_rebuild(generation, mutations).await
    }

    pub async fn prepare_rebuild(&self, generation: i64) -> Result<(), SearchProjectionError> {
        if generation <= 1 {
            return Err(SearchProjectionError::Permanent(
                "SEARCH_GENERATION_INVALID",
            ));
        }
        self.bootstrap().await?;
        for kind in [SearchSourceKind::Published, SearchSourceKind::Draft] {
            let index = self.generation_index(kind, generation);
            let response = self
                .send(
                    self.client
                        .put(self.url(&index)?)
                        .json(&mapping(self.embedding_dimension)),
                )
                .await?;
            if !response.status().is_success() {
                return Err(status_error(response.status()));
            }
        }
        let actions = [SearchSourceKind::Published, SearchSourceKind::Draft]
            .into_iter()
            .map(|kind| {
                json!({"add":{
                    "index":self.generation_index(kind,generation),
                    "alias":self.rebuild_alias(kind)
                }})
            })
            .collect::<Vec<_>>();
        let response = self
            .send(
                self.client
                    .post(self.url("_aliases")?)
                    .json(&json!({"actions":actions})),
            )
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(status_error(response.status()))
        }
    }

    pub async fn activate_rebuild(
        &self,
        generation: i64,
        mutations: &[ProjectionMutation],
    ) -> Result<(), SearchProjectionError> {
        for mutation in mutations {
            self.apply_to_generation(mutation, generation).await?;
        }
        self.validate_generation(generation, mutations).await?;
        self.swap_aliases(generation).await
    }

    pub async fn abort_rebuild(&self) -> Result<(), SearchProjectionError> {
        let mut actions = Vec::new();
        for kind in [SearchSourceKind::Published, SearchSourceKind::Draft] {
            let alias = self.rebuild_alias(kind);
            let response = self
                .send(self.client.get(self.url(&format!("_alias/{alias}"))?))
                .await?;
            if response.status() == StatusCode::NOT_FOUND {
                continue;
            }
            if !response.status().is_success() {
                return Err(status_error(response.status()));
            }
            let value: Value = response
                .json()
                .await
                .map_err(|_| SearchProjectionError::Transient("SEARCH_RESPONSE_INVALID"))?;
            for index in value.as_object().into_iter().flat_map(|value| value.keys()) {
                actions.push(json!({"remove":{"index":index,"alias":alias}}));
            }
        }
        if actions.is_empty() {
            return Ok(());
        }
        let response = self
            .send(
                self.client
                    .post(self.url("_aliases")?)
                    .json(&json!({"actions":actions})),
            )
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(status_error(response.status()))
        }
    }

    async fn bootstrap_kind(&self, kind: SearchSourceKind) -> Result<(), SearchProjectionError> {
        let read = self.read_alias(kind);
        let response = self.send(self.client.head(self.url(&read)?)).await?;
        if response.status().is_success() {
            return Ok(());
        }
        if response.status() != StatusCode::NOT_FOUND {
            return Err(status_error(response.status()));
        }
        let index = self.generation_index(kind, 1);
        let create = self
            .send(
                self.client
                    .put(self.url(&index)?)
                    .json(&mapping(self.embedding_dimension)),
            )
            .await?;
        if !create.status().is_success() && create.status() != StatusCode::BAD_REQUEST {
            return Err(status_error(create.status()));
        }
        let aliases = json!({"actions":[
            {"add":{"index":index,"alias":read}},
            {"add":{"index":index,"alias":self.write_alias(kind),"is_write_index":true}}
        ]});
        let response = self
            .send(self.client.post(self.url("_aliases")?).json(&aliases))
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(status_error(response.status()))
        }
    }

    async fn replace(
        &self,
        target: &str,
        workspace_id: Uuid,
        document_id: Uuid,
        kind: SearchSourceKind,
        sequence: i64,
        regions: &[SearchProjection],
    ) -> Result<(), SearchProjectionError> {
        self.tombstone_matching(
            target,
            workspace_id,
            sequence,
            json!({"term":{"document_id":document_id}}),
        )
        .await?;
        let marker = tombstone_projection(workspace_id, document_id, kind, sequence);
        let mut values = Vec::with_capacity(regions.len() + 1);
        values.push(marker);
        values.extend_from_slice(regions);
        self.bulk_upsert(target, workspace_id, &values).await
    }

    async fn delete_tree(
        &self,
        workspace_id: Uuid,
        root: Uuid,
        sequence: i64,
    ) -> Result<(), SearchProjectionError> {
        for kind in [SearchSourceKind::Published, SearchSourceKind::Draft] {
            self.tombstone_matching(
                &self.write_alias(kind),
                workspace_id,
                sequence,
                json!({"bool":{"should":[
                    {"term":{"document_id":root}},
                    {"term":{"ancestor_ids":root}}
                ],"minimum_should_match":1}}),
            )
            .await?;
        }
        Ok(())
    }

    async fn delete_workspace(
        &self,
        workspace_id: Uuid,
        sequence: i64,
    ) -> Result<(), SearchProjectionError> {
        for kind in [SearchSourceKind::Published, SearchSourceKind::Draft] {
            self.tombstone_matching(
                &self.write_alias(kind),
                workspace_id,
                sequence,
                json!({"match_all":{}}),
            )
            .await?;
        }
        Ok(())
    }

    async fn tombstone_matching(
        &self,
        alias: &str,
        workspace_id: Uuid,
        sequence: i64,
        target: Value,
    ) -> Result<(), SearchProjectionError> {
        let body = json!({
            "script": {
                "lang":"painless",
                "source":"if (ctx._source.outbox_sequence <= params.sequence) { ctx._source.deleted = true; ctx._source.body = ''; ctx._source.terms = []; ctx._source.embedding = null; ctx._source.outbox_sequence = params.sequence; } else { ctx.op = 'noop'; }",
                "params":{"sequence":sequence}
            },
            "query":{"bool":{"filter":[
                {"term":{"workspace_id":workspace_id}},
                target,
                {"range":{"outbox_sequence":{"lte":sequence}}}
            ]}}
        });
        let url = self.url(&format!(
            "{alias}/_update_by_query?conflicts=proceed&refresh=true&routing={workspace_id}"
        ))?;
        let response = self.send(self.client.post(url).json(&body)).await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(status_error(response.status()))
        }
    }

    async fn bulk_upsert(
        &self,
        alias: &str,
        workspace_id: Uuid,
        projections: &[SearchProjection],
    ) -> Result<(), SearchProjectionError> {
        let mut body = String::new();
        for projection in projections {
            let id = projection_id(
                projection.workspace_id,
                parse_kind(&projection.source_kind)?,
                projection.document_id,
                projection.region_id,
            );
            body.push_str(
                &serde_json::to_string(
                    &json!({"update":{"_index":alias,"_id":id,"routing":workspace_id}}),
                )
                .map_err(|_| SearchProjectionError::Permanent("SEARCH_PROJECTION_INVALID"))?,
            );
            body.push('\n');
            body.push_str(
                &serde_json::to_string(&json!({
                    "scripted_upsert":true,
                    "script":{
                        "lang":"painless",
                        "source":"if (ctx._source == null || ctx._source.outbox_sequence <= params.sequence) { ctx._source = params.document; } else { ctx.op = 'noop'; }",
                        "params":{"sequence":projection.outbox_sequence,"document":projection}
                    },
                    "upsert":projection
                }))
                .map_err(|_| SearchProjectionError::Permanent("SEARCH_PROJECTION_INVALID"))?,
            );
            body.push('\n');
        }
        let response = self
            .send(
                self.client
                    .post(self.url("_bulk?refresh=true")?)
                    .header("content-type", "application/x-ndjson")
                    .body(body),
            )
            .await?;
        let status = response.status();
        if !status.is_success() {
            return Err(status_error(status));
        }
        let result: Value = response
            .json()
            .await
            .map_err(|_| SearchProjectionError::Transient("SEARCH_RESPONSE_INVALID"))?;
        if result.get("errors").and_then(Value::as_bool) == Some(false) {
            Ok(())
        } else {
            Err(SearchProjectionError::Transient("SEARCH_BULK_PARTIAL"))
        }
    }

    async fn apply_to_generation(
        &self,
        mutation: &ProjectionMutation,
        generation: i64,
    ) -> Result<(), SearchProjectionError> {
        match mutation {
            ProjectionMutation::Replace {
                workspace_id,
                document_id,
                source_kind,
                sequence,
                regions,
            } => {
                self.replace(
                    &self.generation_index(*source_kind, generation),
                    *workspace_id,
                    *document_id,
                    *source_kind,
                    *sequence,
                    regions,
                )
                .await
            }
            ProjectionMutation::DeleteTree {
                workspace_id,
                root_document_id,
                sequence,
            } => {
                for kind in [SearchSourceKind::Published, SearchSourceKind::Draft] {
                    self.tombstone_matching(
                        &self.generation_index(kind, generation),
                        *workspace_id,
                        *sequence,
                        json!({"bool":{"should":[
                            {"term":{"document_id":root_document_id}},
                            {"term":{"ancestor_ids":root_document_id}}
                        ],"minimum_should_match":1}}),
                    )
                    .await?;
                }
                Ok(())
            }
            ProjectionMutation::DeleteWorkspace {
                workspace_id,
                sequence,
            } => {
                for kind in [SearchSourceKind::Published, SearchSourceKind::Draft] {
                    self.tombstone_matching(
                        &self.generation_index(kind, generation),
                        *workspace_id,
                        *sequence,
                        json!({"match_all":{}}),
                    )
                    .await?;
                }
                Ok(())
            }
        }
    }

    async fn mirror_to_rebuild(
        &self,
        mutation: &ProjectionMutation,
    ) -> Result<(), SearchProjectionError> {
        match mutation {
            ProjectionMutation::Replace { source_kind, .. } => {
                let alias = self.rebuild_alias(*source_kind);
                if self
                    .send(self.client.head(self.url(&alias)?))
                    .await?
                    .status()
                    .is_success()
                {
                    self.apply_to_target(mutation, &alias).await?;
                }
            }
            ProjectionMutation::DeleteTree { .. } | ProjectionMutation::DeleteWorkspace { .. } => {
                for kind in [SearchSourceKind::Published, SearchSourceKind::Draft] {
                    let alias = self.rebuild_alias(kind);
                    if self
                        .send(self.client.head(self.url(&alias)?))
                        .await?
                        .status()
                        .is_success()
                    {
                        self.apply_to_target(mutation, &alias).await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn apply_to_target(
        &self,
        mutation: &ProjectionMutation,
        target: &str,
    ) -> Result<(), SearchProjectionError> {
        match mutation {
            ProjectionMutation::Replace {
                workspace_id,
                document_id,
                source_kind,
                sequence,
                regions,
            } => {
                self.replace(
                    target,
                    *workspace_id,
                    *document_id,
                    *source_kind,
                    *sequence,
                    regions,
                )
                .await
            }
            ProjectionMutation::DeleteTree {
                workspace_id,
                root_document_id,
                sequence,
            } => {
                self.tombstone_matching(
                    target,
                    *workspace_id,
                    *sequence,
                    json!({"bool":{"should":[
                        {"term":{"document_id":root_document_id}},
                        {"term":{"ancestor_ids":root_document_id}}
                    ],"minimum_should_match":1}}),
                )
                .await
            }
            ProjectionMutation::DeleteWorkspace {
                workspace_id,
                sequence,
            } => {
                self.tombstone_matching(target, *workspace_id, *sequence, json!({"match_all":{}}))
                    .await
            }
        }
    }

    async fn validate_generation(
        &self,
        generation: i64,
        mutations: &[ProjectionMutation],
    ) -> Result<(), SearchProjectionError> {
        for kind in [SearchSourceKind::Published, SearchSourceKind::Draft] {
            let expected = mutations
                .iter()
                .filter_map(|mutation| match mutation {
                    ProjectionMutation::Replace {
                        source_kind,
                        regions,
                        ..
                    } if *source_kind == kind => Some(regions.len() as i64),
                    _ => None,
                })
                .sum::<i64>();
            let index = self.generation_index(kind, generation);
            let response = self
                .send(
                    self.client
                        .post(self.url(&format!("{index}/_count"))?)
                        .json(&json!({"query":{"term":{"deleted":false}}})),
                )
                .await?;
            if !response.status().is_success() {
                return Err(status_error(response.status()));
            }
            let value: Value = response
                .json()
                .await
                .map_err(|_| SearchProjectionError::Transient("SEARCH_RESPONSE_INVALID"))?;
            if value.get("count").and_then(Value::as_i64) != Some(expected) {
                return Err(SearchProjectionError::Permanent(
                    "SEARCH_REBUILD_COUNT_MISMATCH",
                ));
            }
        }
        for projection in mutations
            .iter()
            .filter_map(|mutation| match mutation {
                ProjectionMutation::Replace { regions, .. } => regions.first(),
                _ => None,
            })
            .take(10)
        {
            let kind = parse_kind(&projection.source_kind)?;
            let id = projection_id(
                projection.workspace_id,
                kind,
                projection.document_id,
                projection.region_id,
            );
            let index = self.generation_index(kind, generation);
            let response = self
                .send(self.client.get(self.url(&format!(
                    "{index}/_doc/{id}?routing={}",
                    projection.workspace_id
                ))?))
                .await?;
            if !response.status().is_success() {
                return Err(SearchProjectionError::Permanent(
                    "SEARCH_REBUILD_SAMPLE_MISMATCH",
                ));
            }
            let value: Value = response
                .json()
                .await
                .map_err(|_| SearchProjectionError::Transient("SEARCH_RESPONSE_INVALID"))?;
            if value["_source"]["snapshot_hash"] != projection.snapshot_hash
                || value["_source"]["permission_fingerprint"] != projection.permission_fingerprint
            {
                return Err(SearchProjectionError::Permanent(
                    "SEARCH_REBUILD_SAMPLE_MISMATCH",
                ));
            }
        }
        Ok(())
    }

    async fn swap_aliases(&self, generation: i64) -> Result<(), SearchProjectionError> {
        let mut actions = Vec::new();
        for kind in [SearchSourceKind::Published, SearchSourceKind::Draft] {
            let read = self.read_alias(kind);
            let write = self.write_alias(kind);
            let rebuild = self.rebuild_alias(kind);
            let response = self
                .send(
                    self.client
                        .get(self.url(&format!("_alias/{read},{write},{rebuild}"))?),
                )
                .await?;
            if response.status().is_success() {
                let value: Value = response
                    .json()
                    .await
                    .map_err(|_| SearchProjectionError::Transient("SEARCH_RESPONSE_INVALID"))?;
                for index in value.as_object().into_iter().flat_map(|value| value.keys()) {
                    actions.push(json!({"remove":{"index":index,"alias":read}}));
                    actions.push(json!({"remove":{"index":index,"alias":write}}));
                    actions
                        .push(json!({"remove":{"index":index,"alias":rebuild,"must_exist":false}}));
                }
            } else if response.status() != StatusCode::NOT_FOUND {
                return Err(status_error(response.status()));
            }
            let index = self.generation_index(kind, generation);
            actions.push(json!({"add":{"index":index,"alias":read}}));
            actions.push(json!({"add":{"index":index,"alias":write,"is_write_index":true}}));
        }
        let response = self
            .send(
                self.client
                    .post(self.url("_aliases")?)
                    .json(&json!({"actions":actions})),
            )
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(status_error(response.status()))
        }
    }

    async fn send(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, SearchProjectionError> {
        let request = if let Some(credential) = &self.credential {
            request.basic_auth(&credential.username, Some(&credential.password))
        } else {
            request
        };
        request
            .send()
            .await
            .map_err(|_| SearchProjectionError::Transient("SEARCH_UNAVAILABLE"))
    }

    fn url(&self, path: &str) -> Result<Url, SearchProjectionError> {
        self.endpoint
            .join(path)
            .map_err(|_| SearchProjectionError::Permanent("SEARCH_CONFIG_INVALID"))
    }

    fn read_alias(&self, kind: SearchSourceKind) -> String {
        format!("{}-{}-read", self.prefix, kind_name(kind))
    }

    fn write_alias(&self, kind: SearchSourceKind) -> String {
        format!("{}-{}-write", self.prefix, kind_name(kind))
    }

    fn rebuild_alias(&self, kind: SearchSourceKind) -> String {
        format!("{}-{}-rebuild", self.prefix, kind_name(kind))
    }

    fn generation_index(&self, kind: SearchSourceKind, generation: i64) -> String {
        format!(
            "{}-{}-v{}-{generation}",
            self.prefix,
            kind_name(kind),
            SEARCH_PROJECTION_SCHEMA
        )
    }
}

impl SearchIndex for OpenSearchIndex {
    fn apply<'a>(
        &'a self,
        mutations: &'a [ProjectionMutation],
    ) -> BoxFuture<'a, Result<(), SearchProjectionError>> {
        Box::pin(async move {
            self.bootstrap().await?;
            for mutation in mutations {
                match mutation {
                    ProjectionMutation::Replace {
                        workspace_id,
                        document_id,
                        source_kind,
                        sequence,
                        regions,
                    } => {
                        self.replace(
                            &self.write_alias(*source_kind),
                            *workspace_id,
                            *document_id,
                            *source_kind,
                            *sequence,
                            regions,
                        )
                        .await?;
                    }
                    ProjectionMutation::DeleteTree {
                        workspace_id,
                        root_document_id,
                        sequence,
                    } => {
                        self.delete_tree(*workspace_id, *root_document_id, *sequence)
                            .await?
                    }
                    ProjectionMutation::DeleteWorkspace {
                        workspace_id,
                        sequence,
                    } => self.delete_workspace(*workspace_id, *sequence).await?,
                }
                self.mirror_to_rebuild(mutation).await?;
            }
            Ok(())
        })
    }
}

fn tombstone_projection(
    workspace_id: Uuid,
    document_id: Uuid,
    kind: SearchSourceKind,
    sequence: i64,
) -> SearchProjection {
    SearchProjection {
        projection_schema: SEARCH_PROJECTION_SCHEMA,
        workspace_id,
        document_id,
        document_status: "DELETED".to_owned(),
        source_kind: kind.as_str().to_owned(),
        source_revision: 0,
        version_number: None,
        region_id: TOMBSTONE_REGION_ID,
        region_kind: "TOMBSTONE".to_owned(),
        ancestor_ids: Vec::new(),
        title: String::new(),
        body: String::new(),
        terms: Vec::new(),
        embedding: None,
        permission_scope: permission_scope_token(workspace_id, document_id),
        permission_fingerprint: "deleted".to_owned(),
        snapshot_hash: "deleted".to_owned(),
        authority: "NONE".to_owned(),
        updated_at: Utc::now(),
        outbox_sequence: sequence,
        deleted: true,
    }
}

fn mapping(dimension: u32) -> Value {
    json!({
        "settings":{
            "index":{"number_of_shards":3,"number_of_replicas":1,"knn":true},
            "analysis":{"analyzer":{"adoc_ko_en":{"type":"custom","tokenizer":"nori_tokenizer","filter":["lowercase","nori_readingform"]}}}
        },
        "mappings":{"dynamic":"strict","properties":{
            "projection_schema":{"type":"integer"},"workspace_id":{"type":"keyword"},
            "document_id":{"type":"keyword"},"document_status":{"type":"keyword"},
            "source_kind":{"type":"keyword"},"source_revision":{"type":"long"},
            "version_number":{"type":"long"},"region_id":{"type":"keyword"},
            "region_kind":{"type":"keyword"},"ancestor_ids":{"type":"keyword"},
            "title":{"type":"text","analyzer":"adoc_ko_en","fields":{"raw":{"type":"keyword","ignore_above":500}}},
            "body":{"type":"text","analyzer":"adoc_ko_en"},"terms":{"type":"keyword"},
            "embedding":{"type":"knn_vector","dimension":dimension,"method":{"name":"hnsw","space_type":"cosinesimil","engine":"lucene"}},
            "permission_scope":{"type":"keyword"},"permission_fingerprint":{"type":"keyword"},
            "snapshot_hash":{"type":"keyword"},"authority":{"type":"keyword"},
            "updated_at":{"type":"date"},"outbox_sequence":{"type":"long"},"deleted":{"type":"boolean"}
        }}
    })
}

fn kind_name(kind: SearchSourceKind) -> &'static str {
    match kind {
        SearchSourceKind::Published => "published",
        SearchSourceKind::Draft => "draft",
    }
}

fn parse_kind(value: &str) -> Result<SearchSourceKind, SearchProjectionError> {
    match value {
        "PUBLISHED" => Ok(SearchSourceKind::Published),
        "DRAFT" => Ok(SearchSourceKind::Draft),
        _ => Err(SearchProjectionError::Permanent(
            "SEARCH_PROJECTION_INVALID",
        )),
    }
}

fn status_error(status: StatusCode) -> SearchProjectionError {
    if status == StatusCode::NOT_FOUND {
        SearchProjectionError::Transient("SEARCH_ALIAS_CHANGED")
    } else if status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
        SearchProjectionError::Transient("SEARCH_UNAVAILABLE")
    } else {
        SearchProjectionError::Permanent("SEARCH_REQUEST_REJECTED")
    }
}
