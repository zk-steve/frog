def apply_default(args):
    return {
        "image": "quickwit/quickwit:latest",
        "name": "quickwit",
        "http_port": 7280,
        "grpc_port": 7281,
    } | args


def start(plan, args, suffix):
    args = apply_default(args)
    ports = {}
    ports["http"] = PortSpec(
        number=args["http_port"],
        application_protocol="http",
        wait=None,
    )
    ports["grpc"] = PortSpec(
        number=args["grpc_port"],
        application_protocol="http",
        wait=None,
    )

    name = args["name"] + suffix

    service = plan.add_service(
        name=name,
        config=ServiceConfig(
            image=args["image"],
            ports=ports,
            cmd=["run"],
            env_vars={
                "QW_ENABLE_OTLP_ENDPOINT": "true",
                "QW_ENABLE_JAEGER_ENDPOINT": "true",
            },
        ),
    )
