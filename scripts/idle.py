import statistics
import subprocess
import time
import argparse

samples = []

parser = argparse.ArgumentParser()
parser.add_argument("--pid", type=int, required=True)
args = parser.parse_args()

for _ in range(60):
    print("Processing...")
    output = subprocess.check_output(
        ["ps", "-p", str(args.pid), "-o", "rss="],
        text=True,
    )
    samples.append(int(output.strip()) / 1024)
    time.sleep(1)

print(f"Median idle RSS: {statistics.median(samples):.2f} MiB")
print(f"Peak idle RSS:   {max(samples):.2f} MiB")
