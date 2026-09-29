import os
import json
import time
import sys
import signal
import subprocess
import statistics
from typing import List


target: int = int(sys.argv[1])
started = None
focused = None
ready = False

def start_timer(*_):
    global started
    if not ready or focused == target:
        print("Move to your starting window first", flush=True)
        return
    started = time.perf_counter()
    print("Timer started...", flush=True)

signal.signal(signal.SIGUSR1, start_timer)

process = subprocess.Popen(
            ["niri", "msg", "--json", "event-stream"],
            stdout=subprocess.PIPE,
            text=True
        )

assert process.stdout is not None

try:
    results: List[float] = []
    trials = 10
    count = 1
    for line in process.stdout:
        event = json.loads(line)

        #initial 'WindowsChanged' that shows all info on windows
        if "WindowsChanged" in event:
            windows = event["WindowsChanged"]["windows"]
            focused = next(
                        (w["id"] for w in windows if w["is_focused"]), None
                    )
            if not ready:
                ready = True
                print(f"Ready. Start timer with command: kill -USR1 {os.getpid()}", flush=True)

        elif "WindowFocusChanged" in event:
            focused = event["WindowFocusChanged"]["id"]
        
        if started is not None and focused == target:
            elapsed: float = time.perf_counter() - started
            started = None
            print(f"Trial {count} - Reached destination: {elapsed:.3f} seconds", flush=True)
            results.append(elapsed)
            count += 1
            if count == trials:
                break

    print("Time trials complete\n")
    print("\n".join(str(t) for t in results))
    print(f"Average: {statistics.mean(results)}")

except KeyboardInterrupt:
    pass
finally:
    process.terminate()
    process.wait()
