def apply_default(args):
    return {
        "image": "jaegertracing/jaeger-query:latest",
        "name": "jaeger",
        "http_port": 16686,
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

    quickwit_service = plan.get_service(name="quickwit" + suffix)
    quickwit_port = quickwit_service.ports["grpc"].number
    quickwit_address = quickwit_service.ip_address
    service = plan.add_service(
        name=name,
        config=ServiceConfig(
            image=args["image"],
            ports=ports,
            env_vars={
                "SPAN_STORAGE_TYPE": "grpc",
                "GRPC_STORAGE_SERVER": "{}:{}".format(quickwit_address, quickwit_port),
            },
        ),
    )
