#!/usr/bin/env python3
"""Untimed launcher-RSS control; invoke in a fresh Python process per case."""
import json
import os
import resource
import subprocess
import sys

allocation = int(sys.argv[1])
retained = bytearray(allocation)
for offset in range(0, allocation, 4096):
    retained[offset] = 1
with open(os.devnull, 'wb') as output:
    child = subprocess.Popen([sys.argv[2], '--help'], stdout=output, stderr=output)
    _, status, usage = os.wait4(child.pid, 0)
    child.returncode = os.waitstatus_to_exitcode(status)
assert child.returncode == 0
print(json.dumps({'retained_launcher_bytes': allocation,
                  'launcher_peak_rss_kib': resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
                  'child_peak_rss_kib': usage.ru_maxrss, 'exit_code': child.returncode}))
