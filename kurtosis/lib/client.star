def apply_default(args):
    return {
        "image": "zksteve/frog-client:latest",
        "name": "frog-client",
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

    name = args["name"] + suffix

    custom_config_tpl = read_file(src="../templates/01-client.toml")
    custom_config = plan.render_templates(
        name="01-client.toml" + suffix,
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