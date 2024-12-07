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
        // Set up a postgres database port for testing
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
                ("PG__URL".to_string(), database_url),
                ("PG__MAX_SIZE".to_string(), "10".to_string()),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use e2e_tests::utils::get_free_port;
    use reqwest::Client;
    use test_log::test;
    use tokio::time::sleep;

    use super::*;

    #[test(tokio::test)]
    async fn test_full_flow() {
        let setup_config = Setup::new().await;

        let mut server_envs = setup_config.envs.clone();
        let crs_seed = "crs_seed_32_bytes_123456789_123456789_123456789".to_string();
        server_envs.append(&mut vec![
            ("SERVICE_NAME".to_string(), "server".to_string()),
            ("PHANTOM_SERVER__CRS_SEED".to_string(), crs_seed.clone()),
            (
                "PHANTOM_SERVER__PARTICIPANT_NUMBER".to_string(),
                "2".to_string(),
            ),
            ("WORKER__SCHEMA".to_string(), "worker".to_string()),
            (
                "EXPORTER_ENDPOINT".to_string(),
                "127.0.0.1:3000".to_string(),
            ),
        ]);
        let mut server = Program::run("SERVER".to_string(), "frog_server", server_envs);
        server.wait_till_started().await;

        let server_endpoint = format!("http://{}:{}", server.url, server.port);

        let mut client_ports = vec![];
        for _ in 0..2 {
            client_ports.push(get_free_port());
        }

        let mut clients_info = vec![];
        for i in 0..2 {
            let mut client_ports = client_ports.clone();
            let port = client_ports.remove(i);
            let mut client_endpoints = client_ports
                .into_iter()
                .enumerate()
                .map(|(i, e)| {
                    (
                        format!("CLIENT__PEER_ENDPOINTS__{}", i),
                        format!("http://127.0.0.1:{}", e),
                    )
                })
                .collect::<Vec<_>>();

            let mut client_envs = vec![];
            client_envs.append(&mut vec![
                ("SERVICE_NAME".to_string(), format!("client_{}", i)),
                (
                    "CLIENT__SERVER_ENDPOINT".to_string(),
                    server_endpoint.clone(),
                ),
                ("CLIENT__CLIENT_ID".to_string(), i.to_string()),
                ("CLIENT__CRS_SEED".to_string(), crs_seed.clone()),
                ("CLIENT__CLIENT_SEED".to_string(), format!("client_{}", i)),
                (
                    "EXPORTER_ENDPOINT".to_string(),
                    "127.0.0.1:3000".to_string(),
                ),
            ]);
            client_envs.append(&mut client_endpoints);
            let mut client =
                Program::run_with_port(format!("CLIENT_{}", i), "frog_client", client_envs, port);
            client.wait_till_started().await;
            clients_info.push(client);
        }

        // Wait for the result to settle
        sleep(Duration::from_secs(30)).await;

        let max_tries = 400;
        let client = Client::new();
        for _ in 0..max_tries {
            let mut success_count = 0;
            for client_port in client_ports.clone() {
                let response = client
                    .get(format!("http://127.0.0.1:{}/result", client_port))
                    .send()
                    .await
                    .unwrap();
                let result: Option<u64> = response.json().await.unwrap();
                if result.is_some_and(|r| r == 12) {
                    success_count += 1;
                }
            }
            if success_count == client_ports.len() {
                break;
            }
            sleep(Duration::from_secs(1)).await;
        }
    }
}
