use anyhow::bail;
use async_trait::async_trait;

use crate::{
    models::BuildDefinition,
    parser::{Pipeline, PipelineProducer},
};

#[derive(Debug, Default)]
pub struct DefaultPipelineProducer;

#[async_trait]
impl PipelineProducer for DefaultPipelineProducer {
    async fn produce(definition: BuildDefinition) -> anyhow::Result<Pipeline> {
        let mut pipeline = Pipeline::new(definition.project_root, definition.env);

        // add nodes
        for (name, step) in &definition.pipeline {
            pipeline.add_step(name, step.clone());
        }

        // add dependecy between nodes
        let mut has_dependency = false;
        for (name, step) in &definition.pipeline {
            for dependency in &step.depends_on {
                pipeline.depends_on(name, dependency);
                has_dependency = true;
            }
        }

        // if no explicit dependencies, just have prev depend on next
        if !has_dependency {
            let names: Vec<_> = definition.pipeline.keys().collect();
            for window in names.windows(2) {
                pipeline.depends_on(window[1], window[0]);
            }
        }

        // check if there are cyclics, fail the pipeline is so
        if pipeline.is_cyclic() {
            bail!("Screw this pipeline, its cyclic!");
        }

        Ok(pipeline)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::StepDefinition;

    use super::*;
    use indexmap::{IndexMap, indexmap};
    use std::{collections::HashMap, path::PathBuf};

    fn mock_step(image: &str, deps: Vec<&str>) -> StepDefinition {
        StepDefinition {
            image: image.to_string(),
            commands: vec!["echo hello".to_string()],
            depends_on: deps.into_iter().map(|s| s.to_string()).collect(),
            env: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_sequential_fallback_ordering() -> anyhow::Result<()> {
        let pipeline_map = indexmap! {
            "A".to_string() => mock_step("clean", vec![]),
            "B".to_string() => mock_step("build", vec![]),
            "C".to_string() => mock_step("cleanup", vec![]),
        };

        let definition = BuildDefinition {
            pipeline: pipeline_map,
            project_root: PathBuf::from("/tmp"),
            env: HashMap::new(),
        };

        let mut pipeline = DefaultPipelineProducer::produce(definition).await?;

        assert!(!pipeline.is_cyclic(),);

        let a = pipeline.ids.get("A").unwrap();
        let b = pipeline.ids.get("B").unwrap();
        let c = pipeline.ids.get("C").unwrap();

        assert!(pipeline.graph.contains_edge(*a, *b));
        assert!(pipeline.graph.contains_edge(*b, *c));
        Ok(())
    }

    #[tokio::test]
    async fn test_explicit_dependencies_override_fallback() -> anyhow::Result<()> {
        let pipeline_map = indexmap! {
            "A".to_string() => mock_step("alpine", vec![]),
            "B".to_string() => mock_step("alpine", vec![]),
            "C".to_string() => mock_step("alpine", vec!["A"]),
        };

        let definition = BuildDefinition {
            pipeline: pipeline_map,
            project_root: PathBuf::from("/tmp"),
            env: HashMap::new(),
        };

        let mut pipeline = DefaultPipelineProducer::produce(definition).await?;

        assert!(!pipeline.is_cyclic());
        let a = pipeline.ids.get("A").unwrap();
        let b = pipeline.ids.get("B").unwrap();
        let c = pipeline.ids.get("C").unwrap();

        println!("{:#?}", pipeline.graph);

        assert!(pipeline.graph.contains_edge(*a, *c));
        assert!(!pipeline.graph.contains_edge(*c, *a));
        assert!(!pipeline.graph.contains_edge(*a, *b));
        assert!(!pipeline.graph.contains_edge(*b, *a));
        Ok(())
    }

    #[tokio::test]
    async fn test_cyclic_dependency_error() -> anyhow::Result<()> {
        let pipeline_map = indexmap! {
            "node_a".to_string() => mock_step("alpine", vec!["node_b"]),
            "node_b".to_string() => mock_step("alpine", vec!["node_a"]),
        };

        let definition = BuildDefinition {
            pipeline: pipeline_map,
            project_root: PathBuf::from("/"),
            env: HashMap::new(),
        };

        let result = DefaultPipelineProducer::produce(definition).await;

        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("cyclic"),
            "Error message should mention cycle: {}",
            err_msg
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_empty_pipeline() -> anyhow::Result<()> {
        let definition = BuildDefinition {
            pipeline: IndexMap::new(),
            project_root: PathBuf::from("/"),
            env: HashMap::new(),
        };

        let result = DefaultPipelineProducer::produce(definition).await;

        assert!(result.is_ok());
        assert!(result.unwrap().ids.is_empty());
        Ok(())
    }
}
