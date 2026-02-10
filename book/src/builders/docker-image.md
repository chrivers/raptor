# Docker Image generator:<br>**`docker-image`**

| Mount name                | Type   | Usage                                                                                        |
|:--------------------------|:-------|:---------------------------------------------------------------------------------------------|
| `input`                   | Layers | The Raptor build target to genrate as a docker image.                                        |
| `output`                  | File   | Points to the resulting docker image file.                                                   |
| `cache`<br>**(Optional)** | Simple | Cache for tar files of built layers.<br>Will save time when repeating layers between builds. |

This builder generates a Docker image from a Raptor target.

Each layer in Raptor (one for the target, plus one for each `FROM`) becomes a
Docker layer in the generated image. This allows shared Raptor layers to stay
shared when converted to Docker images.

## Compatibility

| Target                      | Compatible? |
|:----------------------------|:------------|
| Container: `systemd-nspawn` | ❌          |
| Container: `portablectl`    | ❌          |
| Container: `docker`         | ✅          |
| Container: `podman`         | ✅          |
| Virtual Machine (UEFI)      | ❌          |
| Virtual Machine (BIOS)      | ❌          |
| Physical Machine (UEFI)     | ❌          |
| Physical Machine (BIOS)     | ❌          |

## Example

Prerequisites:

 - [raptor-builders](https://github.com/chrivers/raptor-builders) is cloned to `raptor-builders`
 - An input target called `test.rapt`

~~~admonish note title="Raptor.toml"
```toml
[raptor.link]
rbuild = "raptor-builders"

# Docker image
[run.docker1]
target = "$rbuild.docker-image" # <-- builder is specified here
input = ["test"]
output = "test.tar"
## to speed up builds with shared layers,
## specify a cache directory:
# cache = "/tmp/docker-image-cache"
```
~~~

After this `Raptor.toml` is in place, call `raptor make` to build:

```sh
sudo raptor make docker1
```

When the process is complete, `test.tar` will be ready for use.

It can be imported:

```sh
# import to Docker
docker load -i test.tar

# import to Podman
podman load -i test.tar
```
