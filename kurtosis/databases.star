# We support both local and remote Postgres databases within our Kurtosis package
# When 'USE_REMOTE_POSTGRES' is False, service automatically creates all databases locally
# When 'USE_REMOTE_POSTGRES' is True, service is created just as a helper for param injection across pods
# When 'USE_REMOTE_POSTGRES' is True, all state is stored on your preconfigured remote Postgres instances
USE_REMOTE_POSTGRES = False

# When 'USE_REMOTE_POSTGRES' is True, replace 'POSTGRES_HOSTNAME' with your master database IP/hostname
POSTGRES_HOSTNAME = "127.0.0.1"

# Mostly static params unless user has specialized postgres configuration
POSTGRES_IMAGE = "postgres:16.2-alpine"
POSTGRES_SERVICE_NAME = "postgres"
POSTGRES_PORT = 5432

# Below 'POSTGRES_MASTER_' params only apply when 'USE_REMOTE_POSTGRES' is False
POSTGRES_MASTER_DB = "master"
POSTGRES_MASTER_USER = "master_user"
POSTGRES_MASTER_PASSWORD = "master_password"

# When 'USE_REMOTE_POSTGRES' is True, update following credentials to match your remote postgres DBs
# It is recommended users keep existing DB names and usernames for stability

FROG_DBS = {
    "frog_db": {
        "name": "frog_db_db",
        "user": "frog_db_user",
        "password": "redacted",
    },
}

DATABASES = FROG_DBS


def run(plan, suffix):
    db_configs = get_db_configs(suffix)
    create_postgres_service(plan, db_configs, suffix)


def get_db_configs(suffix):
    dbs = DATABASES

    configs = {
        k: v
        | {
            "hostname": POSTGRES_HOSTNAME
            if USE_REMOTE_POSTGRES
            else _service_name(suffix),
            "port": POSTGRES_PORT,
        }
        for k, v in dbs.items()
    }
    return configs


def _service_name(suffix):
    return POSTGRES_SERVICE_NAME + suffix


def create_postgres_service(plan, db_configs, suffix):
    init_script_tpl = read_file(src="./templates/init-db.sql")
    init_script = plan.render_templates(
        name="init.sql" + suffix,
        config={
            "init.sql": struct(
                template=init_script_tpl,
                data={
                    "dbs": db_configs,
                    "master_db": POSTGRES_MASTER_DB,
                    "master_user": POSTGRES_MASTER_USER,
                },
            )
        },
    )

    postgres_service_cfg = ServiceConfig(
        image=POSTGRES_IMAGE,
        ports={
            "postgres": PortSpec(POSTGRES_PORT, application_protocol="postgresql"),
        },
        env_vars={
            "POSTGRES_DB": POSTGRES_MASTER_DB,
            "POSTGRES_USER": POSTGRES_MASTER_USER,
            "POSTGRES_PASSWORD": POSTGRES_MASTER_PASSWORD,
        },
        files={"/docker-entrypoint-initdb.d/": init_script},
        cmd=["-N 1000"],
    )

    plan.add_service(
        name=_service_name(suffix),
        config=postgres_service_cfg,
        description="Starting Postgres Service",
    )
