import os
import pexpect
import sys
import subprocess
import tempfile
from argparse import ArgumentParser, REMAINDER
from pexpect import fdpexpect
from subprocess import Popen
from vncdotool import api

def main():
    parser = ArgumentParser()
    parser.add_argument('--screenshot', action='store_true', default=False)
    parser.add_argument('qemu_cmd', nargs=REMAINDER)
    args = parser.parse_args()
    fifo_name = os.path.join(tempfile.mkdtemp(), 'fifo')
    os.mkfifo(fifo_name)

    try:
        child_args = args.qemu_cmd + ['-serial', 'pipe:' + fifo_name, '-vnc', ':0']
        print('> %s' % subprocess.list2cmdline(child_args))
        with Popen(child_args) as child_proc:
            print('[test.py] Started process %d' % child_proc.pid)
            try:

                with open(fifo_name, 'rb') as fifo:
                    child = fdpexpect.fdspawn(fifo, encoding='utf8', logfile=sys.stdout, timeout=10)
                    result = child.expect([r'\[kernel\] end kmain|System ready', r'\[kernel::unwind\] (.*)', pexpect.TIMEOUT])
                    if result == 0:
                        print('[test.py] Success')
                    elif result == 1:
                        (message,) = child.match.groups()
                        print('[test.py] Failed: %s' % message)
                    elif result == 2:
                        print('[test.py] Timed out')

            finally:
                if args.screenshot:
                    client = api.connect('localhost:0', password=None)
                    filename = 'screenshot.png'
                    print('[test.py] Saving screenshot to %s' % filename)

                    prev_screenshot_bytes = None

                    if result == 0:
                        try:
                            with open(filename, 'rb') as f:
                                prev_screenshot_bytes = f.read()
                        except:
                            pass

                    client.captureScreen(filename)

                    if prev_screenshot_bytes is not None:
                        with open(filename, 'rb') as f:
                            screenshot_bytes = f.read()

                        if prev_screenshot_bytes != screenshot_bytes:
                            result = 3

                print('[test.py] Stopping process %d' % child_proc.pid)
                child_proc.kill()
                print('[test.py] Waiting for process %d to exit... ' % child_proc.pid, end='', flush=True)
                child_proc.wait()
                print('done')

    finally:
        os.unlink(fifo_name)

    return result

if __name__ == '__main__':
    result = main()
    exit(result)
