#!/usr/bin/env python
	
import requests
import json
url = "http://localhost:18443"
headers = {"content-type": "application/json; charset=UTF-8"}

def send_json(payload):
	response = requests.post(url, headers=headers, data=json.dumps(payload))
	response_native = json.loads(response.text)
	return response_native

def gen_keypair():
	payload = {"jsonrpc": "2.0", "method": "generatekeypair", "params": [], "id":1 } 
	return  send_json(payload).get('result')

def mine_block(addresshash, num_block):
	payload = {"jsonrpc": "2.0", "method": "generateblocks", "params": [addresshash, num_block], "id":1 }
	return send_json(payload).get('result')

def add_tx_to_wallet(txid, out_index):
	payload = {"jsonrpc": "2.0", "method": "walletaddtx", "params": [txid, out_index], "id":1 }
	send_json(payload)

def update_wallet():
	payload = {"jsonrpc": "2.0", "method": "getspendable", "params": [], "id":1 }
	send_json(payload)

def print_blocks():
	payload = {"jsonrpc": "2.0", "method": "print_blocks", "params": [], "id":1 }
	send_json(payload)

def get_balance():
	payload = {"jsonrpc": "2.0", "method": "getbalance", "params": [], "id":1 }
	send_json(payload)

def generate_key_and_init_fund():
	address_hash = gen_keypair()
	txid = mine_block(address_hash, 1) # gen 1 block
	print_blocks()
	add_tx_to_wallet(txid, 0) # first tx
	update_wallet()
	get_balance()

generate_key_and_init_fund()
#print(address_hash)
