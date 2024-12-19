# How to Run

## Using Kurtosis

### Prerequisites

Before starting, ensure you have the following tools installed:

- [Docker](https://docs.docker.com/get-docker/) (
  version >= [4.27.0](https://docs.docker.com/desktop/release-notes/#4270) for Mac users)
- [Kurtosis](https://docs.kurtosis.com/install/)

### Deployment

Once the prerequisites are installed, run the following commands to deploy the complete stack locally.

```bash
cd kurtosis
kurtosis clean --all
kurtosis run --enclave frog-v1 --args-file params.yaml .
```

You should see an output similar to this:

```
...Omitted...

========================================== User Services ==========================================
UUID           Name                Ports                                                Status
403d5c31dc9d   frog-client-0-001   http: 9944/tcp -> http://127.0.0.1:32982             RUNNING
982dacc80e6f   frog-client-1-001   http: 9944/tcp -> http://127.0.0.1:32983             RUNNING
9daf9e37b994   frog-server-001     http: 9944/tcp -> http://127.0.0.1:32980             RUNNING
145281e8f2eb   frog-worker-001     http: 9944/tcp -> http://127.0.0.1:32981             RUNNING
202b5db0b6d1   jaeger-001          http: 16686/tcp -> http://127.0.0.1:32979            RUNNING
be800261692b   postgres-001        postgres: 5432/tcp -> postgresql://127.0.0.1:32976   RUNNING
3de7d5c6cea0   quickwit-001        grpc: 7281/tcp -> http://127.0.0.1:32978             RUNNING
                                   http: 7280/tcp -> http://127.0.0.1:32977        
```

To view container logs, use the `docker logs` command.

After a few minutes, you can query the result from the client with the following command (note: replace the port with
what you have):

```bash
curl http://127.0.0.1:32982/result
```

---

## For Developers

### Prerequisites

Ensure you have a running PostgreSQL instance on port 5432. If your PostgreSQL instance is running on a different port,
update the configuration files accordingly.

### Deployment

- Start the server:

```bash
cd crates/public
RUST_BACKTRACE=1 RUST_LOG=info cargo run --
```

- In another terminal, start the worker:

```bash
cd crates/worker
RUST_BACKTRACE=1 RUST_LOG=info cargo run --
```

- In another terminal, start the first client:

```bash
cd crates/client
RUST_BACKTRACE=1 RUST_LOG=info cargo run --
```

- In another terminal, start the second client with a specific configuration:

```bash
cd crates/client
RUST_BACKTRACE=1 RUST_LOG=info cargo run -- -c ./config/01-client-01.toml
```

Once all components are running, you can test the system as required.
