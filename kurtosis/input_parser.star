GLOBAL_LOG_LEVEL = struct(
    error="error",
    warn="warn",
    info="info",
    debug="debug",
    trace="trace",
)

DEFAULT_ARGS = {
    "deployment_suffix": "-001",
    "global_log_level": "info",
}

def parse_args(args):
    args = DEFAULT_ARGS | args
    validate_global_log_level(args["global_log_level"])
    return args


def validate_global_log_level(global_log_level):
    if global_log_level not in (
        GLOBAL_LOG_LEVEL.error,
        GLOBAL_LOG_LEVEL.warn,
        GLOBAL_LOG_LEVEL.info,
        GLOBAL_LOG_LEVEL.debug,
        GLOBAL_LOG_LEVEL.trace,
    ):
        fail(
            "Unsupported global log level: '{}', please use '{}', '{}', '{}', '{}' or '{}'".format(
                global_log_level,
                GLOBAL_LOG_LEVEL.error,
                GLOBAL_LOG_LEVEL.warn,
                GLOBAL_LOG_LEVEL.info,
                GLOBAL_LOG_LEVEL.debug,
                GLOBAL_LOG_LEVEL.trace,
            )
        )