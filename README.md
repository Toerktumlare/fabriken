# Fabriken

A distributed build execution system written in Rust.

Fabriken is a toy learning project i wrote to explor distributed build systems through a
controller/agent architecture, gRPC communication, DAG-based execution, and
containerized build workloads.

The system separates **build orchestration** from **build execution**:

- The **Control Plane** manages pipelines, agents, and build dispatching.
- **Agents** execute builds by constructing DAGs and running tasks inside isolated containers.

## Architecture

```
                    +-------------+
                    |   User/API  |
                    +-------------+
                           |
                           | HTTP REST
                           v
                 +-------------------+
                 |   Control Plane   |
                 |                   |
                 |  Pipeline Parser  |
                 |  Build Scheduler |
                 |  Agent Registry  |
                 +-------------------+
                    |             ^
                    | gRPC        |
                    |             | log stream
                    v             |
          +-----------------------------+
          |          Agent              |
          |                             |
          |  Build Specification        |
          |          |                  |
          |          v                  |
          |     DAG Construction        |
          |          |                  |
          |          v                  |
          |     Task Execution          |
          |          |                  |
          |          v                  |
          |       Podman                |
          |    +---------+              |
          |    | Build 1 |              |
          |    +---------+              |
          |    | Build 2 |              |
          |    +---------+              |
          +-----------------------------+

          Agent Registration
                  |
                  v
          +----------------+
          | gRPC Health    |
          | Service        |
          +----------------+
```

## Workflow

### 1. Build submission

A user starts a build by sending a request to the Control Plane REST API.

The request contains the location of the project containing the pipeline definition.

Example:

```
POST /build

{
    "path": "/projects/example"
}
```

The Control Plane then:

1. Reads the pipeline YAML file
2. Parses the build configuration
3. Creates a build specification
4. Dispatches the build to an available agent

---

### 2. Agent lifecycle

Agents automatically register when they start.

The Control Plane maintains agent state using gRPC health checking.

This allows the system to track:

- Available agents
- Agent failures
- Agent lifecycle changes

```
Agent Startup
      |
      v
+-------------+
|   Agent     |
+-------------+
      |
      | Register
      v
+-------------+
| Control     |
| Plane       |
+-------------+
      |
      | Health Checks
      v
+-------------+
| gRPC Health |
+-------------+
```

---

### 3. Build execution

Once an agent receives a build specification:

1. The agent constructs a DAG representing build dependencies.
2. Tasks are scheduled according to dependency order.
3. Individual tasks are executed inside Podman containers.
4. Build output and logs are streamed back to the Control Plane using gRPC.

Example pipeline:

```
             Build
               |
       +-------+-------+
       |               |
    Compile          Test
       |
       v
    Package
```

---

## Features

- Rust-based distributed build execution
- Controller / agent architecture
- REST API build triggering
- gRPC communication
- Automatic agent registration
- gRPC health monitoring
- YAML-defined pipelines
- DAG-based task scheduling
- Podman container execution
- Streaming build logs
