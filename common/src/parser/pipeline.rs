use anyhow::bail;
use async_trait::async_trait;

use crate::{
    models::BuildDefinition,
    parser::{GlobalData, Pipeline, PipelineProducer},
};

#[derive(Debug, Default)]
pub struct DefaultPipelineProducer;

#[async_trait]
impl PipelineProducer for DefaultPipelineProducer {
    async fn produce(definition: BuildDefinition) -> anyhow::Result<(GlobalData, Pipeline)> {
        let project_root = definition.context.project_root;
        let global_data = GlobalData {
            project_root,
            env: definition.env,
        };

        let mut pipeline = Pipeline::new();

        // add nodes
        for step in definition.pipeline.iter() {
            pipeline.add_step(step.clone());
        }

        // add dependecy between nodes
        let mut has_dependency = false;
        for step in &definition.pipeline {
            for dependency in &step.depends_on {
                pipeline.depends_on(&step.name, dependency);
                has_dependency = true;
            }
        }

        // if no explicit dependencies, just have prev depend on next
        if !has_dependency {
            let names: Vec<_> = definition.pipeline.iter().map(|step| &step.name).collect();
            for window in names.windows(2) {
                pipeline.depends_on(window[1], window[0]);
            }
        }

        // check if there are cyclics, fail the pipeline is so
        if pipeline.is_cyclic() {
            bail!("Screw this pipeline, its cyclic!");
        }

        Ok((global_data, pipeline))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Context, Step};

    use super::*;
    use std::{collections::HashMap, path::PathBuf};

    fn mock_step(image: &str, deps: Vec<&str>) -> Step {
        Step {
            name: image.to_string(),
            image: Some(image.to_string()),
            run: vec!["echo hello".to_string()],
            depends_on: deps.into_iter().map(|s| s.to_string()).collect(),
            env: HashMap::new(),
            containerize: None,
            push: None,
        }
    }

    #[tokio::test]
    async fn test_sequential_fallback_ordering() -> anyhow::Result<()> {
        let pipeline_map = vec![
            mock_step("A", vec![]),
            mock_step("B", vec![]),
            mock_step("C", vec![]),
        ];

        let definition = BuildDefinition {
            pipeline: pipeline_map,
            env: HashMap::new(),
            context: Context {
                project_root: PathBuf::from("/tmp"),
            },
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
        let pipeline_map = vec![
            mock_step("A", vec![]),
            mock_step("B", vec![]),
            mock_step("C", vec!["A"]),
        ];

        let definition = BuildDefinition {
            pipeline: pipeline_map,
            env: HashMap::new(),
            context: Context {
                project_root: PathBuf::from("/tmp"),
            },
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
        let pipeline_map = vec![
            mock_step("node_a", vec!["node_b"]),
            mock_step("node_b", vec!["node_a"]),
        ];

        let definition = BuildDefinition {
            pipeline: pipeline_map,
            env: HashMap::new(),
            context: Context {
                project_root: PathBuf::from("/tmp"),
            },
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
            pipeline: vec![],
            env: HashMap::new(),
            context: Context {
                project_root: PathBuf::from("/tmp"),
            },
        };

        let result = DefaultPipelineProducer::produce(definition).await;

        assert!(result.is_ok());
        assert!(result.unwrap().ids.is_empty());
        Ok(())
    }
}
