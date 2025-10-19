# Raptor make

## Overall structure

Below is an example of the overall structure of a `Raptor.toml` file.

**Note**: All parts are optional, so you only need to define the parts you need.

```toml
[raptor.link]
name1 = "path/to/source1"
name2 = "path/to/source2"
...

[run.purple]
# ..run target here..

[run.orange]
# ..run target here..

[group.green]
# ..group here..

[group.yellow]
# ..group here..
```

## Run target format

A run target (`[run.<name>]`) is the most commonly used feature in
`Raptor.toml`.

The structure for a job named `example` is shown below, where each field is
specified with its default value.

**Note**: Only the `target` field is required! Everything else can be specified as needed.

~~~admonish note title="Raptor.toml run"
```toml
[run.example]
target = <required>

# Cache mounts
# (default is empty list)
#
# Note: A single element can be specified as a string instead of the list
cache = []

# Input mounts
# (default is empty list)
#
# Note: A single element can be specified as a string instead of the list
input = []

# Output mounts
# (default is empty list)
#
# Note: A single element can be specified as a string instead of the list
output = []

# Entrypoint arguments
entrypoint = []

# Command arguments
args = []

# BTreeMap<String, String>
env = {}

# State directory for container state
# (default is unset, meaning ephemeral containers)
#state_dir =
```
~~~

## Group format

A group is used to collectively refer to a number of run and build jobs, by a
single name.

```toml
[group.example]
# List of layers to build
#
# Default is empty list
build = []

# List of names for [run.<name>] targets to run
#
# Default is empty list
run = []
```
