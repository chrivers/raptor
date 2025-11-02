# Instruction `CMD`

~~~admonish summary
```raptor
CMD <command> [...<arg>]
```
~~~

```admonish important title="Build-time instruction"
The `CMD` instructions only affects *running* a container, not *building* a
container.
```

This instruction sets the default command for the container, which is used when
running commands in it.

Unlike `ENTRYPOINT`, this instruction does not have a default value.

The semantics of the `ENTRYPOINT` and `CMD` instructions are heavily inspired by
Docker.

~~~admonish info title="About the Docker design...",collapsible=true
Many people find the Docker design of `ENTRYPOINT` and `CMD` confusing, and
unneccessarily complicated. So why duplicate it in Raptor?

We considered other options, but in the end, a redesign would have meant more
complexity for people familiar with Docker, and the risk of confusing the
semantics would increase.

So we are leaning towards what is familiar to some people, even though Raptor
does not have the [distinction
between](https://docs.docker.com/reference/dockerfile/#shell-and-exec-form)
"shell form" and "exec form" that Docker has.
~~~

When running a container, the **command** will be run through the **entrypoint**.

The **command** can be specified on the command line. If no command is given,
the `CMD` instruction provides the fallback value. If no `CMD` instruction is
found either, the run will fail with an error.

The **entrypoint** is taken from the `ENTRYPOINT` instruction, if
present. Otherwise, the default value is used.

## Example

As an example, let us consider a simple Raptor container:

~~~admonish file title="ping.rapt"
```raptor
{{#include ../../example/ping.rapt}}
```
~~~

This container will install the `iptutils-ping` package, containing the `ping`
command. Since `/bin/ping` is set as the entrypoint, the container can be
invoked to ping destinations:

```sh
$ sudo raptor run -L book book/example/ '$book.ping' 1.1.1.1
[*] Completed [675DE2C3A4D8CD82] index.docker.io-library-debian-trixie
[*] Completed [20D13F4B24A66D8B] ping
PING 1.1.1.1 (1.1.1.1) 56(84) bytes of data.
64 bytes from 1.1.1.1: icmp_seq=1 ttl=52 time=13.9 ms
64 bytes from 1.1.1.1: icmp_seq=2 ttl=52 time=15.2 ms
64 bytes from 1.1.1.1: icmp_seq=3 ttl=52 time=11.8 ms
^C
--- 1.1.1.1 ping statistics ---
3 packets transmitted, 3 received, 0% packet loss, time 2003ms
rtt min/avg/max/mdev = 11.762/13.613/15.190/1.412 ms
```

However, if no arguments are provided, we fall back to `8.8.8.8` (specified with
the `CMD` instruction):

```sh
$ sudo raptor run -L book book/example/ '$book.ping'
[*] Completed [675DE2C3A4D8CD82] index.docker.io-library-debian-trixie
[*] Completed [20D13F4B24A66D8B] ping
PING 8.8.8.8 (8.8.8.8) 56(84) bytes of data.
64 bytes from 8.8.8.8: icmp_seq=1 ttl=117 time=9.04 ms
64 bytes from 8.8.8.8: icmp_seq=2 ttl=117 time=9.24 ms
^C
--- 8.8.8.8 ping statistics ---
2 packets transmitted, 2 received, 0% packet loss, time 1000ms
rtt min/avg/max/mdev = 9.039/9.137/9.235/0.098 ms
```
