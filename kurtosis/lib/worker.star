def apply_default(args):
    return {
        "image": "zksteve/frog-worker:latest",
        "name": "frog-worker",
        "http_port": 9944,
        "concurrent": 10,
    } | args


def start(plan, args, suffix):
    args = apply_default(args)
    ports = {}
    ports["http"] = PortSpec(
        number=args["http_port"],
        application_protocol="http",
        wait=None,
    )

    name = args["name"] + suffix

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

    custom_config_tpl = read_file(src="../templates/01-worker.toml")
    custom_config = plan.render_templates(
        name="01-worker.toml" + suffix,
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
