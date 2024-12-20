def apply_default(args):
    return {
        "image": "zksteve/frog-server:latest",
        "name": "frog-server",
        "crs_seed": "crs_seed_32_bytes_123456789_123456789_123456789",
        "participant_number": 2,
        "http_port": 9944,
    } | args


def start(plan, args, suffix):
    args = apply_default(args)
    ports = {}
    ports["http"] = PortSpec(
        number=args["http_port"],
        application_protocol="http",
        wait=None,
    )

    name = get_server_name(args, suffix)
    postgres_service = plan.get_service(name="postgres" + suffix)
    postgres_port = postgres_service.ports["postgres"].number
    postgres_address = postgres_service.ip_address
    frog_db = import_module("../databases.star").FROG_DBS["frog_db"]
    args["pg_url"] = "postgres://{}:{}@{}:{}/{}".format(
        frog_db["user"],
        frog_db["password"],
        postgres_address,
        postgres_port,
        frog_db["name"],
    )
    args["schema"] = frog_db["name"]

    custom_config_tpl = read_file(src="../templates/01-server.toml")
    custom_config = plan.render_templates(
        name="01-server.toml" + suffix,
        config={
            "01-custom.toml": struct(
                template=custom_config_tpl,
                data=args,
            )
        },
    )

    service = plan.add_service(
        name=name,
        config=ServiceConfig(
            image=args["image"],
            ports=ports,
            cmd=[],
            files={"/user/": custom_config},
        ),
    )


def get_server_name(args, suffix):
    args = apply_default(args)
    return args["name"] + suffix
