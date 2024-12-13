server_package = "./server.star"
server_lib_package = "./lib/server.star"
worker_package = "./worker.star"
client_package = "./client.star"

databases_package = "./databases.star"

input_parser = "./input_parser.star"

# Additional services packages.
grafana_package = "./additional_services/grafana.star"
prometheus_package = "./additional_services/prometheus.star"
quickwit_package = "./additional_services/quickwit.star"
jaeger_package = "./additional_services/jaeger.star"


def run(
    plan,
    deploy_server=True,
    deploy_worker=True,
    deploy_client=True,
    deploy_databases=True,
    deploy_quickwit=True,
    args={},
):
    args = import_module(input_parser).parse_args(args)
    args = args | {"deploy_quickwit": deploy_quickwit}
    args = args | {"deploy_server": deploy_server}
    args = args | {"deploy_worker": deploy_worker}
    args = args | {"deploy_client": deploy_client}
    args = args | {"deploy_databases": deploy_databases}
    plan.print("Deploying with parameters: " + str(args))

    # Deploy databases.
    if args["deploy_databases"]:
        plan.print("Deploying databases")
        import_module(databases_package).run(
            plan,
            suffix=args["deployment_suffix"],
        )
    else:
        plan.print("Skipping the deployment of databases")

    exporter_endpoint = "http://0.0.0.0:7281"

    if args["deploy_quickwit"]:
        plan.print("Deploying quickwit")
        import_module(quickwit_package).start(
            plan,
            {},
            suffix=args["deployment_suffix"],
        )

        plan.print("Deploying jaeger")
        import_module(jaeger_package).start(
            plan,
            {},
            suffix=args["deployment_suffix"],
        )

        quickwit_service = plan.get_service(name="quickwit" + args["deployment_suffix"])
        quickwit_port = quickwit_service.ports["grpc"].number
        quickwit_address = quickwit_service.ip_address
        exporter_endpoint = "http://{}:{}".format(
            quickwit_address,
            quickwit_port,
        )
    else:
        plan.print("Skipping the deployment of quickwit")

    if args["deploy_server"]:
        plan.print("Deploying server")
        args["server"]["exporter_endpoint"] = exporter_endpoint
        import_module(server_package).run(
            plan,
            args["server"],
            suffix=args["deployment_suffix"],
        )
    else:
        plan.print("Skipping the deployment of the server")

    if args["deploy_worker"]:
        plan.print("Deploying workers")
        args["worker"]["exporter_endpoint"] = exporter_endpoint
        import_module(worker_package).run(
            plan,
            args["worker"],
            suffix=args["deployment_suffix"],
        )
    else:
        plan.print("Skipping the deployment of workers")

    if args["deploy_client"]:
        plan.print("Deploying clients")
        args["client"]["server_name"] = import_module(
            server_lib_package
        ).get_server_name(args["server"], suffix=args["deployment_suffix"])
        args["client"]["exporter_endpoint"] = exporter_endpoint
        import_module(client_package).run(
            plan, args["client"], suffix=args["deployment_suffix"]
        )
    else:
        plan.print("Skipping the deployment of clients")
