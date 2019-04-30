#!/usr/bin/env python3

import socket
import subprocess

local_url = "http://127.0.0.1:18443"

def get_my_ip():
    line = ""
    with open('/etc/hosts') as f:
        for l in f:
            line = l
    my_ip = line.split()[0]
    return my_ip

HOST = get_my_ip()
print('HOST', HOST)
PORT = 65432

def send_json(url, payload):
    response = session.post(url, headers=headers, data=json.dumps(payload), verify=False)
    response_native = json.loads(response.text)
    return response_native

def add_tx_to_wallet(url, txid, out_index):
    payload = {"jsonrpc": "2.0", "method": "walletaddtx", "params": [txid, out_index], "id":1 }
    send_json(url, payload)


while 1:
	with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
		s.bind((HOST, PORT))
		s.listen()
		conn, addr = s.accept()
		with conn:
			print('Connected by', addr)
			while True:
				data = conn.recv(1024).split()
				txid, out_index = data[0], data[1]	
				
				print(data)
				if not data:
					break
				add_tx_to_wallet(local_url, txid, out_index)
				conn.sendall('Y')
