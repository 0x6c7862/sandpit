#!/usr/bin/env python3
# This is free and unencumbered software released into the public domain.
#
# Anyone is free to copy, modify, publish, use, compile, sell, or
# distribute this software, either in source code form or as a compiled
# binary, for any purpose, commercial or non-commercial, and by any
# means.
#
# In jurisdictions that recognize copyright laws, the author or authors
# of this software dedicate any and all copyright interest in the
# software to the public domain. We make this dedication for the benefit
# of the public at large and to the detriment of our heirs and
# successors. We intend this dedication to be an overt act of
# relinquishment in perpetuity of all present and future rights to this
# software under copyright law.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
# EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
# MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
# IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
# OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
# ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
# OTHER DEALINGS IN THE SOFTWARE.
#
# For more information, please refer to <http://unlicense.org>
"""Script to assist in debugging the sandbox process. Runs a shared library.

On a modern distro run this as `gdb --python ./share/inject/src/python.py`.
Otherwise, try `gdb -x `./share/inject/src/python.py` or using the `source`
command. If your distro's gdb doesn't support Python at all maybe consider
trying a different one :)
"""
import base64
import gdb
import shutil
import subprocess
import os
import sys
from distutils import sysconfig


def main():
    payload = "/tmp/payload.so"

    # Break on main in the sandbox
    gdb.execute('file target/release/sandpit')
    gdb.execute('set follow-fork-mode child')
    gdb.execute('b main')
    gdb.execute('r')  # main() in broker
    gdb.execute('c')  # main() in sandbox
    gdb.execute('d 1')

    # Make lib directories
    os.makedirs('/tmp/sandpit.sandbox/bin')
    os.makedirs('/tmp/sandpit.sandbox/dev')
    os.makedirs('/tmp/sandpit.sandbox/lib')
    os.makedirs('/tmp/sandpit.sandbox/lib64')
    os.makedirs('/tmp/sandpit.sandbox/sbin')
    os.makedirs('/tmp/sandpit.sandbox/tmp')
    os.makedirs('/tmp/sandpit.sandbox/usr')
    os.makedirs('/tmp/sandpit.sandbox/var')

    # dlopen the target library
    # NOTE: 4354 == RTLD_NOW | RTLD_GLOBAL | RTLD_NODELETE
    gdb.execute('call dlopen("{}", 4354)'.format(payload))

    # Break on mounts
    gdb.execute('b mount')
    gdb.execute('c')  # mount /
    gdb.execute('c')  # remount .
    gdb.execute('c')  # mount proc

    # Bind mount some additional directories for convenience
    # NOTE: 20480 == MS_REC | MS_BIND
    gdb.execute('d 2')
    gdb.execute('call mount("/bin", "bin", 0, 20480, 0)')
    gdb.execute('call mount("/dev/", "dev", 0, 20480, 0)')
    gdb.execute('call mount("/lib/", "lib", 0, 20480, 0)')
    gdb.execute('call mount("/lib64/", "lib64", 0, 20480, 0)')
    gdb.execute('call mount("/sbin", "sbin", 0, 20480, 0)')
    gdb.execute('call mount("/usr", "usr", 0, 20480, 0)')

    # Break on the initial IPC write
    gdb.execute('b prctl')
    gdb.execute('c')  # drop new privs
    gdb.execute('c')  # drop capabilities
    gdb.execute('d 3')
    gdb.execute('b write') # "Reading an allowed file"
    gdb.execute('c')  # "Reading an allowed file...
    gdb.execute('d 4')

    # Run the payload
    gdb.execute('call payload(4)')
    gdb.execute('p errno')


if __name__ == '__main__':
    main()
