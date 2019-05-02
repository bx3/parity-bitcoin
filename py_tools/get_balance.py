#!/usr/bin/env python
import sys
import send_json as sender
from wallet_basic import get_balance

if __name__ == '__main__':
    num_args = len(sys.argv)

    if num_args==1:
        url = "http://127.0.0.1:18443"
        get_balance(url)
    elif num_args==2:
        ip = sys.argv[1]
        url = "http://" + ip + ":" + "18443"
        get_balance(url)
    elif num_args==3:
        ip, port = sys.argv[1], sys.argv[2]
        url = "http://" + ip + ":" + port
        get_balance(url)
    else:
        print("check args: [ip] or [ip port]")
