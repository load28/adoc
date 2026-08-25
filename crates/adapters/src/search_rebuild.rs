use chrono::{DateTime, Utc};

use adoc_application::search::SearchProjectionError;

use crate::{postgres::PostgresSearchProjectionRepository, search_index::OpenSearchIndex};

pub struct SearchRebuilder {
    repository: PostgresSearchProjectionRepository,
    index: OpenSearchIndex,
}

impl SearchRebuilder {
    #[must_use]
    pub fn new(repository: PostgresSearchProjectionRepository, index: OpenSearchIndex) -> Self {
        Self { repository, index }
    }

    pub async fn run(&self, now: DateTime<Utc>) -> Result<i64, SearchProjectionError> {
        let run = self.repository.begin_rebuild(now).await?;
        if let Err(error) = self.index.prepare_rebuild(run.generation).await {
            let _ = self
                .repository
                .finish_rebuild(run.id, Some(error_code(error)), now)
                .await;
            return Err(error);
        }
        let mutations = match self.repository.capture_rebuild_snapshot(&run, now).await {
            Ok(mutations) => mutations,
            Err(error) => {
                let _ = self.index.abort_rebuild().await;
                let _ = self
                    .repository
                    .finish_rebuild(run.id, Some(error_code(error)), now)
                    .await;
                return Err(error);
            }
        };
        match self
            .index
            .activate_rebuild(run.generation, &mutations)
            .await
        {
            Ok(()) => {
                self.repository.finish_rebuild(run.id, None, now).await?;
                Ok(run.generation)
            }
            Err(error) => {
                let _ = self.index.abort_rebuild().await;
                let _ = self
                    .repository
                    .finish_rebuild(run.id, Some(error_code(error)), now)
                    .await;
                Err(error)
            }
        }
    }
}

fn error_code(error: SearchProjectionError) -> &'static str {
    match error {
        SearchProjectionError::Transient(code) | SearchProjectionError::Permanent(code) => code,
    }
}
