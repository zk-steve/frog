def apply_default(args):
    return {
        "image": "zksteve/frog-client:latest",
        "name": "frog-client",
        "crs_seed": "crs_seed_32_bytes_123456789_123456789_123456789",
        "session_id": "f8e774bd-2f9d-4502-92ca-ac8b9c25868e",
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

    server_service = plan.get_service(name=args["server_name"])
    server_port = server_service.ports["http"].number
    server_address = server_service.ip_address
    args["server_endpoint"] = "http://{}:{}".format(
        server_address,
        server_port,
    )
    custom_config_tpl = read_file(src="../templates/01-client.toml")

    def get_endpoint(i):
        return '{} = "http://{}:{}"'.format(
            i, get_client_name(args["name"], i, suffix), args["http_port"]
        )

    all_peer_endpoints = [get_endpoint(i) for i in range(args["participants"])]

    for id in range(args["participants"]):
        cloned_args = {} | args
        cloned_args["client_id"] = id
        cloned_args = {
            "client_seed": "client{}_seed_32_bytes_123456789_123456789".format(id)
        } | cloned_args

        peer_endpoints = all_peer_endpoints[:]
        peer_endpoints.pop(id)

        peer_endpoints_str = ",".join(peer_endpoints)
        peer_endpoints_str = "{" + peer_endpoints_str + "}"

        cloned_args["peer_endpoints"] = peer_endpoints_str

        custom_config = plan.render_templates(
            name="01-client-{}.toml{}".format(id, suffix),
            config={
                "01-custom.toml": struct(
                    template=custom_config_tpl,
                    data=cloned_args,
                )
            },
        )

        service_name = get_client_name(args["name"], id, suffix)
        service = plan.add_service(
            name=service_name,
            config=ServiceConfig(
                image=args["image"],
                ports=ports,
                cmd=[],
                files={"/user/": custom_config},
            ),
        )


def get_client_name(name, id, suffix):
    return "{}-{}{}".format(name, id, suffix)
