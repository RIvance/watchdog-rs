# watchdog-rs
A simple watchdog for linux process

```
USAGE:
    watchdog [OPTIONS] <EXECUTABLE> [ARGS]...

ARGS:
    <EXECUTABLE>    Executable file path
    <ARGS>...       List of arguments to pass to the executable which should be seperated by
                    delimitator "--"

OPTIONS:
    -e, --stderr <STDERR>    Redirect stderr
    -h, --help               Print help information
    -i, --stdin <STDIN>      Redirect stdin
    -o, --stdout <STDOUT>    Redirect stdout
    -t, --delay <DELAY>      Restart delay (millionsecond) [default: 1000]
    -V, --version            Print version information
```
