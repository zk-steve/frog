# How To Run

> [!NOTE]  
> This section is under construction and may contain inaccuracies.

To get started, execute the following command in your terminal:

```shell
./frog --help
```

This will display the available options for running the server:

```
Frog

Usage: frog [OPTIONS] [COMMAND]

Commands:
  config  Print config
  help    Print this message or the help of the given subcommand(s)

Options:
  -c, --config-path <CONFIG_PATH>  Config file [default: config/default.toml]
  -v, --version                    Print version
  -h, --help                       Print help
```

## Example

- Multiple config locations

```shell
./cli -c ./config/*.toml -c deploy/local/custom.toml
```

- Pipe the output with [bunyan](https://github.com/trentm/node-bunyan)

```shell
cargo install bunyan
./cli -c ./config/*.toml -c deploy/local/custom.toml | bunyan
```

# Configuration

## Order of apply

Configuration is applied in the following order: config files -> environment variables -> command-line arguments.

If you use `-c *.toml` to load config files, please be mindful of the order in which the files are applied.

### Environment Variable Examples

The server can be configured using environment variables. Below is a table outlining the available configuration
options:

Hierarchical child config via env, separated by using `__`. Specify list values by using `,` separator

| ENV                                                                      | DEFAULT VALUE | NOTE      |
|--------------------------------------------------------------------------|---------------|-----------|
| [RUST_LOG](https://docs.rs/env_logger/latest/env_logger/) > LOG\_\_LEVEL | "INFO"        | Log level |
| SERVER\_\_URL                                                            |               |           |
| SERVER\_\_PORT                                                           |               |           |
| SERVICE_NAME                                                             |               |           |
| EXPORTER_ENDPOINT                                                        |               |           |
| DB\_\_PG\_\_URL                                                          | "localhost"   |           |
| DB\_\_PG\_\_MAX_SIZE                                                     | 5432          |           |

Make sure to set these environment variables according to your needs before running the server.
