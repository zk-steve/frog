client_package = import_module("./lib/client.star")


def run(plan, args, suffix):
    client_package.start(plan, args, suffix)
