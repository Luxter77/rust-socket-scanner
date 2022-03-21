# it scans ip ranges for open ports from the file extra/connlist.csv

tries to open a socket to that port on that ip.

ports, timeouts and number of threads are hardcoded because.

# FOLDER: 
- ./extra/

## csv format:

- no headder.
- from_ip, to_ip

## output

- csv
- ip, port, UTF-8 headder

go figure