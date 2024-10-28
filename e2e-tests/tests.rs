use e2e_tests::postgres::Postgres;
use e2e_tests::program::Program;
use log::info;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ContainerAsync;

#[allow(dead_code)]
/// Initial setup for e2e tests
struct Setup {
    pub postgres_instance: ContainerAsync<Postgres>,
    pub envs: Vec<(String, String)>,
}

impl Setup {
    /// Initialise a new setup
    pub async fn new() -> Self {
        // Set up a postgres database question port for testing
        let postgres_instance = Postgres::default().start().await.unwrap();

        let database_url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            postgres_instance.get_host().await.unwrap(),
            postgres_instance.get_host_port_ipv4(5432).await.unwrap()
        );
        info!("✅ PostgresDB setup completed; URL: {}", &database_url);

        Self {
            postgres_instance,
            envs: vec![
                ("WORKER__SCHEMA".to_string(), "postgres".to_string()),
                ("PG__URL".to_string(), database_url),
                ("PG__MAX_SIZE".to_string(), "10".to_string()),
                (
                    "EXPORTER_ENDPOINT".to_string(),
                    "127.0.0.1:3000".to_string(),
                ),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use e2e_tests::utils::get_free_port;
    use test_log::test;
    use tokio::time::sleep;

    use super::*;

    #[test(tokio::test)]
    async fn test_full_flow() {
        let setup_config = Setup::new().await;

        let mut server_envs = setup_config.envs.clone();
        server_envs.append(&mut vec![(
            "SERVICE_NAME".to_string(),
            "server".to_string(),
        )]);
        let mut server = Program::run("SERVER".to_string(), "frog_server", server_envs);
        server.wait_till_started().await;
        let server_endpoint = format!("http://{}:{}", server.url, server.port);

        let mut worker_envs = setup_config.envs.clone();
        worker_envs.append(&mut vec![
            ("SERVICE_NAME".to_string(), "worker".to_string()),
            ("WORKER__CONCURRENT".to_string(), "5".to_string()),
        ]);
        let mut worker = Program::run("WORKER".to_string(), "frog_worker", worker_envs);
        worker.wait_till_started().await;
        let mut client_ports = vec![];
        for _ in 0..4 {
            client_ports.push(get_free_port());
        }

        let mut clients_info = vec![];
        for i in 0..4 {
            let mut client_ports = client_ports.clone();
            let port = client_ports.remove(i);
            let mut player_endpoints = client_ports
                .into_iter()
                .enumerate()
                .map(|(i, e)| {
                    (
                        format!("GAME__PLAYER_ENDPOINTS__{}", i),
                        format!("http://127.0.0.1:{}", e),
                    )
                })
                .collect::<Vec<_>>();

            let mut client_envs = setup_config.envs.clone();
            client_envs.append(&mut vec![
                ("SERVICE_NAME".to_string(), format!("client_{}", i)),
                ("GAME__SERVER_ENDPOINT".to_string(), server_endpoint.clone()),
                ("GAME__PLAYER_ID".to_string(), i.to_string()),
            ]);
            client_envs.append(&mut player_endpoints);
            let mut client =
                Program::run_with_port(format!("CLIENT_{}", i), "frog_client", client_envs, port);
            client.wait_till_started().await;
            clients_info.push(client);
        }

        sleep(Duration::from_secs(10)).await;
    }
}
