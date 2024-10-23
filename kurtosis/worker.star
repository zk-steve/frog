worker_package = import_module("./lib/worker.star")


def run(plan, args, suffix):
    worker_package.start(plan, args, suffix)
