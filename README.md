# Frog

[![License](https://img.shields.io/github/license/zk-steve/frog)](https://github.com/zk-steve/frog/blob/master/LICENSE)

## Introduction

Welcome to **Frog**, a template project for building real-world MPC (Multi-Party Computation) systems using
the [Phantom library](https://github.com/gausslabs/phantom-zone/tree/rewrite).

## How to Run

Step-by-step instructions for running the project are available [here](docs/how_to_run.md).

## Architecture

Learn more about the system's architecture [here](docs/architecture.md).

## Future Plans

- [x] ~~Finalize Docker and Kurtosis configurations to simplify deployment on Docker or Kubernetes.~~
- [x] ~~Add comprehensive documentation and comments in the codebase to improve maintainability and usability.~~
- [ ] Transition Phantom-related parameters from being stored in the database to a dedicated file storage solution
  (e.g., S3) for better performance.
- [x] ~~Complete the implementation of the worker service and database (currently, the server processes tasks and stores
  data locally).~~
- [ ] Develop a toolkit similar to [Ignite](https://github.com/ignite/cli) to streamline the development process with a
  CLI for scaffolding and managing projects.
- [ ] Implement a more efficient method for handling common parameters.
- [ ] Facilitate client communication through the server (using WebSockets) rather than direct interaction, enabling
  easier communication when clients are in different private networks.