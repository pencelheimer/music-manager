mod coordinator;
pub mod db;
mod mock_pipeline;

pub use coordinator::*;
pub use mock_pipeline::*;

use std::{collections::HashMap, sync::Arc};

use core_bin::{Coordinator, init_tracing};
use core_lib::messages::TrackProcessor;

use kameo::prelude::*;
use once_cell::sync::Lazy;
use sqlx::SqlitePool;

static TRACING: Lazy<()> = Lazy::new(|| {
    init_tracing();
});

pub struct TestApp {
    pub db_pool: SqlitePool,
    pub steps: Vec<String>,
    pub pipeline: ActorRef<MockPipeline>,
    pub coordinator: ActorRef<Coordinator>,
}

impl TestApp {
    pub fn new<V: AsRef<[T]>, T: ToString>(db_pool: SqlitePool, steps: V) -> Self {
        Lazy::force(&TRACING);

        let steps: Vec<_> = steps.as_ref().iter().map(|s| s.to_string()).collect();

        let pipeline = MockPipeline::spawn_default();
        let coordinator = Self::mock_coordinator(db_pool.clone(), steps.clone(), pipeline.clone());

        Self {
            db_pool,
            steps,
            pipeline,
            coordinator,
        }
    }

    fn mock_coordinator(
        db_pool: SqlitePool,
        pipeline_steps: Vec<String>,
        pipeline: ActorRef<MockPipeline>,
    ) -> ActorRef<Coordinator> {
        let registered_plugins = Self::mock_plugins(pipeline, pipeline_steps.clone());

        let coordinator = Coordinator {
            db_pool,
            pipeline_steps,
            registered_plugins,
        };

        Coordinator::spawn(coordinator)
    }

    fn mock_plugins(
        self_ref: ActorRef<MockPipeline>,
        steps: Vec<String>,
    ) -> HashMap<String, Arc<dyn TrackProcessor>> {
        steps
            .iter()
            .map(|stage| {
                (
                    stage.clone(),
                    Arc::new(self_ref.clone()) as Arc<dyn TrackProcessor>,
                )
            })
            .collect()
    }

    pub async fn coordinator_send_sync<M: Send + 'static>(&self, msg: M)
    where
        Coordinator: Message<M> + Message<Ping>,
    {
        self.coordinator.tell(msg).send().await.unwrap();
        self.coordinator.ask(Ping).send().await.unwrap();
    }
}
