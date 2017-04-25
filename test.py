import os
import pexpect
import sys
import subprocess
import tempfile
from pexpect import fdpexpect
from subprocess import Popen

if __name__ == '__main__':
    fifo_name = os.path.join(tempfile.mkdtemp(), 'fifo')
    os.mkfifo(fifo_name)

    try:
        child_args = sys.argv[1:] + ['-serial', 'pipe:' + fifo_name]
        print('> %s' % subprocess.list2cmdline(child_args))
        with Popen(child_args) as child_proc:
            print('Started process %d' % child_proc.pid)
            try:
                with open(fifo_name, 'rb') as fifo:
                    child = fdpexpect.fdspawn(fifo, encoding='utf8', logfile=sys.stdout, timeout=10)
                    result = child.expect([r'\[kernel\] end kmain', r'\[kernel::unwind\] (.*)', pexpect.TIMEOUT])
                    if result == 0:
                        print('Success')
                    elif result == 1:
                        (message,) = p.match.groups()
                        print('Failed: %s' % message)
                    elif result == 2:
                        print('Timed out')

            finally:
                print('Stopping process %d' % child_proc.pid)
                child_proc.kill()
                print('Waiting for process %d to exit... ' % child_proc.pid, end='', flush=True)
                child_proc.wait()
                print('done')

    finally:
        os.unlink(fifo_name)

    exit(result)
