# Inject

Two scripts to assist in debugging. Both will load the process under `gdb`, stop
at the appropriate place, mount more directories than the sandbox normally would
and load a library. One script will load a shared library of the user's choice
to execute arbitrary code. The other will load and initialize a Python
interpreter and execute a Python script.


## Usage

Read the source. They're not written very robustly (there's hardcoded paths and
stupid things like that). Treat them more as a guide :)
