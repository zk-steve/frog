server_package = "./server.star"
worker_package = "./worker.star"
client_package = "./client.star"

databases_package = "./databases.star"

input_parser = "./input_parser.star"

# Additional services packages.
grafana_package = "./src/additional_services/grafana.star"
prometheus_package = "./src/additional_services/prometheus.star"

def run(
    plan,
    deploy_server=True,
    deploy_worker=True,
    deploy_client=True,
    deploy_databases=True,
    args={},
):
    args = import_module(input_parser).parse_args(args)
    args = args | {"deploy_server": deploy_server}
    args = args | {"deploy_worker": deploy_worker}
    args = args | {"deploy_client": deploy_client}
    args = args | {"deploy_databases": deploy_databases}
    plan.print("Deploying with parameters: " + str(args))

    # Deploy databases.
    if deploy_databases:
        plan.print("Deploying databases")
        import_module(databases_package).run(
            plan,
            suffix=args["deployment_suffix"],
        )
    else:
        plan.print("Skipping the deployment of databases")

    if args["deploy_server"]:
        plan.print("Deploying server")
        import_module(server_package).run(
            plan,
            args["server"],
            suffix=args["deployment_suffix"],
        )
    else:
        plan.print("Skipping the deployment of the server")

    if args["deploy_worker"]:
        plan.print("Deploying workers")
        import_module(worker_package).run(
            plan,
            args["worker"],
            suffix=args["deployment_suffix"],
        )
    else:
        plan.print("Skipping the deployment of workers")


    if args["deploy_client"]:
        plan.print("Deploying clients")
        import_module(client_package).run(
            plan,
            args["client"],
            suffix=args["deployment_suffix"],
        )
    else:
        plan.print("Skipping the deployment of clients")
