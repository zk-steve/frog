server_package = import_module("./lib/server.star")


def run(plan, args, suffix):
    server_package.start(plan, args, suffix)