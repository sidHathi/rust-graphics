import matplotlib.pyplot as plt
import sys
import numpy as np

if len(sys.argv) < 1:
  print("No log file path provided in exec args")
  exit(1)

screen_width = 0
screen_height = 0
f32_tex_vals = np.array([])
log_file_path = sys.argv[1]

with open(log_file_path, "rb") as f:
  header = f.readline().strip().decode('utf8')

  split_header = header.split(' ')
  if len(split_header) < 4:
    print("invalid header")
    exit(1)
  
  screen_width = int(split_header[1][:-1])
  screen_height = int(split_header[3][:-1])
  byte_data = np.fromfile(f, dtype=np.float32)
  f32_tex_vals = byte_data.tolist()

if screen_height < 1 or screen_width < 1:
  print("Invalid screen dims for analyzer")
  exit(1)

num_occurrences = {}
for val in f32_tex_vals:
  if not num_occurrences.__contains__(round(val, 3)):
    num_occurrences[round(val, 3)] = 0
  num_occurrences[round(val, 3)] += 1

print("header read successfully")
print("occurrence map: " + str(num_occurrences))