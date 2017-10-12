# Sandpit

Sandpit is a toy application to test debugging.

Slides from the presentation can be found
[here](https://github.com/0x6c7862/sandpit/releases/download/v1.0.0/presentation.pdf).


## Challenge

The application will drop 3 flags to disk upon starting. While these can be read
just by `cat`ing them from your shell, the point of the challenge here is to
read these files _from the sandbox process_.

The scenario is not supposed to include the discovery of a remote code execution
vulnerability in the sandbox, and instead is supposed to come from the
assumption that remote code execution has already been attained. This is
simulated by interacting with and instrumenting the sandbox process with
debuggers, code injection, etc. to perform the actions one's payload would
perform after bootstrapping itself in a complete exploitation scenario.

Included in the `share/` directory are two scripts to assist in debugging. Both
will load the process under `gdb`, stop at the appropriate place, mount more
directories than the sandbox normally would and load a library. One script will
load a shared library of the user's choice to execute arbitrary code. The other
will load and initialize a Python interpreter and execute a Python script.

See
[here](https://github.com/0x6c7862/sandpit/blob/master/doc/answers.md)
for answers. If you get stuck, try reading the source of the sandbox first
before reading the answers :)


## Usage

### Running

After [installation](#installation), run the binary as a standard user without
arguments (it doesn't matter where the file is or what your `pwd` is):

```
./sandpit
```

This should spawn the broker process, which will then subsequently spawn the
sandbox. Both process's working directory is `/tmp/sandpit.sandbox`. After
initialization, the processes will connect to each other over Unix domain
sockets and begin sending "ping" IPC requests to each other.

### IPC

The IPC works over a standard Unix domain socket pair. The files are called
"server" and "client" and reside in the sandbox root.

#### Commands

The following commands are defined:

* `ping`
* `open`

All messages are null terminated.

##### Ping

This command does nothing other than to send a message back and forth through
the IPC.

The request should be of the form:

```
ping
```

The response should be of the form:

```
pong
```

##### Open

This command takes the following form, where `/file/name` is a valid UNIX file
path:

```
open /file/name
```

Note the path refers to the real root filesystem, not the sandbox, as the
operation is performed by the broker.

The server does not respond over the IPC and instead uses `iov` to represent
success or failure through the `sendmsg()`/`recvmsg()` APIs.

The policy applied prevents reading from paths which don't start with
`"/tmp/sandpit.sandbox"`.


## Installation

### Release

Download pre-built binaries from [here](https://github.com/0x6c7862/sandpit/releases).

### Source

Install Rust as per the [latest installation instructions](https://www.rust-lang.org).

Next, run the following command:

```bash
cargo build --release
```

The sandpit binary should be found under `target/release/sandpit`.
