#!/usr/bin/env python
	
import requests
import json
import socket
import subprocess
import os
from time import sleep

session = requests.Session()
session.trust_env = False


headers = {"content-type": "application/json; charset=UTF-8"}
ip_list = ["172.25.0.2", "172.25.0.3"]

def get_my_ip():
    line = ""
    with open('/etc/hosts') as f:
        for l in f:
            line = l
    my_ip = line.split()[0]
	# some hack
    if my_ip[0] == 'f':
        my_ip = "127.0.0.1"

    return my_ip

def get_other_ip():
    #all_ips = set(ip_list)
	# single node simulation
    my_ip = get_my_ip()
    if my_ip == "127.0.0.1":
        return my_ip

    for ip in ip_list:
        if ip != my_ip:
            return ip
    print("no ip available")
    return ""



peer_ip = get_other_ip()

# send to which wallet
peer_url = "http://" + peer_ip + ":18443"
local_url = "http://" + get_my_ip()  + ":18443"


def send_json(url, payload):
	response = session.post(url, headers=headers, data=json.dumps(payload), verify=False)
	response_native = json.loads(response.text)
	return response_native

def gen_keypair(url):
	payload = {"jsonrpc": "2.0", "method": "generatekeypair", "params": [], "id":1 } 
	return  send_json(url, payload).get('result')

# return coinbase txid
def mine_block(url, addresshash, num_block):
	payload = {"jsonrpc": "2.0", "method": "generateblocks", "params": [addresshash, num_block], "id":1 }
	return send_json(url, payload).get('result')

def add_tx_to_wallet(url, txid, out_index):
	payload = {"jsonrpc": "2.0", "method": "walletaddtx", "params": [txid, out_index], "id":1 }
	send_json(url, payload)

def update_wallet(url):
	payload = {"jsonrpc": "2.0", "method": "getspendable", "params": [], "id":1 }
	send_json(url, payload)

def print_blocks(url):
	payload = {"jsonrpc": "2.0", "method": "print_blocks", "params": [], "id":1 }
	send_json(url, payload)

def get_balance(url):
	update_wallet(url)
	payload = {"jsonrpc": "2.0", "method": "getbalance", "params": [], "id":1 }
	return send_json(url, payload).get('result')

def get_addresshash(url):
	payload = {"jsonrpc": "2.0", "method": "getaddresshash", "params": [], "id":1 } 
	return  send_json(url, payload).get('result')

def shard_pay(url, addresshash, amount):
	payload = {"jsonrpc": "2.0", "method": "shardpay", "params": [addresshash, amount], "id":1 } 
	return send_json(url, payload).get('result')



def init_addr_with_fund(url):
	address_hash = gen_keypair(url)
	print("my addresshash", address_hash)
	txid = mine_block(url, address_hash, 1) # gen 1 block
	print("mined a block with coinbase txid " + txid)
	#print_blocks(url)
	add_tx_to_wallet(url, txid, 0) # first tx
	update_wallet(url)
	my_balance = get_balance(url)
	print("your balance")
	print(my_balance)

def pay_peer(my_url, peer_url, amount):
	peer_addresshash = ""
	if my_url != peer_url:
		peer_addresshash = get_addresshash(peer_url)
	else:
		peer_addresshash = gen_keypair(my_url)
	
	print('get peer_addrhash', peer_addresshash)
	txid = shard_pay(my_url, peer_addresshash, amount)
	print('pay to peer txid', txid)
	add_tx_to_wallet(peer_url, txid, 0) # first tx
	

		

print("local_url", local_url)
print("peer_url", peer_url)

init_addr_with_fund(local_url)
pay_peer(local_url, peer_url, 3)
print("after pay peer")
print(get_balance(local_url))
sleep(1)
coinbase_id = mine_block(local_url, get_addresshash(local_url), 1)
print_blocks(local_url)
print("after mine a block")
print(get_balance(local_url))
#init_addr_with_fund(local_url)




#init_addr_with_fund(peer_url)

#print(address_hash)
