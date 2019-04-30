#!/usr/bin/env python
import subprocess
print("hello world")
print("second")


subprocess.call(['cargo', 'build', '-p', 'pbtc'])

def get_my_ip():
    line = ""
    with open('/etc/hosts') as f:
        for l in f:
            line = l
    my_ip = line.split()[0]
    return my_ip

interface_token = '--jsonrpc-interface=' + get_my_ip()
pbtc_cmd = ['/build/parity-bitcoin/target/debug/pbtc', '--btc', '--regtest', interface_token]
pbtc_str = " ".join(x for x in pbtc_cmd);
print('run cmd" ' + pbtc_str)

subprocess.call(pbtc_cmd)


