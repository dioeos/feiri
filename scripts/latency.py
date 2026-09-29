import argparse
import json
import queue
import subprocess
import threading
import time
import math
import statistics


def read_events(stdout, events):
    try:
        for line in stdout:
            event = json.loads(line)
            observed_at = time.perf_counter_ns()
            events.put((event, observed_at))
    except Exception as error:
        events.put(error)
    finally:
        events.put(None)


def wait_for_event(events, event_name, timeout, predicate=None):
    deadline = time.perf_counter() + timeout

    while True:
        remaining = deadline - time.perf_counter()
        if remaining <= 0:
            raise TimeoutError()

        try:
            item = events.get(timeout=remaining)
        except queue.Empty:
            raise TimeoutError() from None

        if item is None:
            raise RuntimeError("Niri closed the event stream")
        if isinstance(item, Exception):
            raise RuntimeError(f"Failed to read Niri events: {item}")

        event, observed_at = item

        if event_name in event:
            data = event[event_name]
            if predicate is None or predicate(data):
                return data, observed_at


def measure(target_id, command, timeout):
    events = queue.Queue()
    command_process = None

    event_process = subprocess.Popen(
        ["niri", "msg", "--json", "event-stream"],
        stdout=subprocess.PIPE,
        text=True,
    )

    assert event_process.stdout is not None

    event_thread = threading.Thread(
        target=read_events,
        args=(event_process.stdout, events),
        daemon=True,
    )
    event_thread.start()

    try:
        snapshot, _ = wait_for_event(
            events, "WindowsChanged", timeout
        )

        target = next(
            (
                window
                for window in snapshot["windows"]
                if window["id"] == target_id
            ),
            None,
        )

        if target is None:
            raise RuntimeError(f"Window {target_id} does not exist")

        if target["is_focused"]:
            raise RuntimeError(
                "Target is already focused; focus another window first"
            )

        start = time.perf_counter_ns()

        command_process = subprocess.Popen(
            command,
            stdout=subprocess.DEVNULL,
        )

        _, end = wait_for_event(
            events,
            "WindowFocusChanged",
            timeout,
            predicate=lambda focus: focus["id"] == target_id,
        )

        return_code = command_process.wait(timeout=timeout)
        if return_code != 0:
            raise RuntimeError(
                f"Command exited with status {return_code}"
            )

        elapsed_ms = (end - start) / 1_000_000
        print(f"Command-to-focus latency: {elapsed_ms:.3f} ms")
        return elapsed_ms

    finally:
        if command_process is not None and command_process.poll() is None:
            command_process.kill()
            command_process.wait()

        if event_process.poll() is None:
            event_process.terminate()

        try:
            event_process.wait(timeout=1)
        except subprocess.TimeoutExpired:
            event_process.kill()
            event_process.wait()

        event_thread.join()
        event_process.stdout.close()


parser = argparse.ArgumentParser()
parser.add_argument("--start", type=int, required=True)
parser.add_argument("--target", type=int, required=True)
parser.add_argument("--trials", type=int, default=100)
parser.add_argument("--timeout", type=float, default=5.0)
parser.add_argument("command", nargs=argparse.REMAINDER)
args = parser.parse_args()

command = args.command
if command and command[0] == "--":
    command = command[1:]

if not command:
    parser.error("Provide a command after --")
if args.timeout <= 0:
    parser.error("--timeout must be greater than zero")
if args.trials <= 0:
    parser.error("--trials must be greater than zero")
if args.start == args.target:
    parser.error("--start and --target must be different")

try:
    results = []

    for trial in range(args.trials):
        subprocess.run(
            [
                "niri", "msg", "action", "focus-window",
                "--id", str(args.start),
            ],
            check=True,
            stdout=subprocess.DEVNULL,
            timeout=args.timeout,
        )

        time.sleep(0.3)

        print(
            f"Trial {trial + 1}/{args.trials}: ",
            end="",
            flush=True,
        )

        elapsed_ms = measure(args.target, command, args.timeout)
        results.append(elapsed_ms)

    ordered = sorted(results)
    p95 = ordered[math.ceil(0.95 * len(ordered)) - 1]

    print(f"\nCompleted {len(results)} trials")
    print(f"Mean:   {statistics.mean(results):.3f} ms")
    print(f"Median: {statistics.median(results):.3f} ms")
    print(f"p95:    {p95:.3f} ms")
    print(f"Min:    {min(results):.3f} ms")
    print(f"Max:    {max(results):.3f} ms")

except (TimeoutError, subprocess.TimeoutExpired):
    raise SystemExit("Timed out waiting for Niri or the command")
except (
    RuntimeError,
    OSError,
    KeyError,
    subprocess.CalledProcessError,
) as error:
    raise SystemExit(str(error))
except KeyboardInterrupt:
    raise SystemExit("\nBenchmark interrupted")
