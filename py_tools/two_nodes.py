#!/usr/bin/env python

import requests
import json
import socket
import subprocess
import os
from time import sleep
from wallet_basic import *

headers = {"content-type": "application/json; charset=UTF-8"}
ip_list = ["172.18.0.2", "172.18.0.3"] #, "172.25.0.4"

def init_addr_with_fund(url):
    address_hash = gen_keypair(url)
    print("my addresshash", address_hash)
    txid = mine_block(url, address_hash, 1) # gen 1 block
    print("mined a block with coinbase txid " + txid)
    wallet_add_tx(url, txid, 0) # first tx
    update_wallet(url)

def pay_peer(my_url, peer_url, amount):
    peer_addresshash = ""
    if my_url != peer_url:
        peer_addresshash = gen_keypair(peer_url)
    else:
        peer_addresshash = gen_keypair(my_url)

    print('get peer_addrhash', peer_addresshash)
    update_wallet(my_url)
    txid = shard_pay(my_url, peer_addresshash, amount)
    print('pay to peer txid', txid)
    wallet_add_tx(peer_url, txid, 0) # first tx

def print_all_balance(all_url):
    for url in all_url:
        balance = get_balance(url)
        print('account', url, balance)

def get_url(ip):
	return "http://" + ip + ":18443"

local_ip = get_my_ip()
peers_ip = get_other_ips(ip_list)

local_url = get_url(local_ip)
peers_url = [get_url(peer_ip) for peer_ip in peers_ip]
all_url = [local_url] + peers_url

assert(len(peers_url) > 0); assert(len(local_url) > 0)

print("local_ip", local_ip)
print("peers_ip", peers_ip)

#give one node so,emoney
init_addr_with_fund(local_url)
init_addr_with_fund(local_url)
print_all_balance(all_url)
print_coins(local_url)

for peer_url in peers_url:
    # send to which wallet
    pay_peer(local_url, peer_url, 4)
    # note coin cannot beimmediately used
    #pay_peer(local_url, peer_url, 2)
    sleep(1)
    coinbase_txid = mine_block(local_url, get_addresshash(local_url), 1)
    #print_all_balance(all_url)
    print('Local_url_coins')
    print_coins(local_url)
    print('Peer_url_coins')
    print_coins(peer_url)
    #pay_peer(local_url, peer_url, 2)
    sleep(1)
    coinbase_id = mine_block(local_url, get_addresshash(local_url), 1)

    sleep(2)
    coinbase_id = mine_block(local_url, get_addresshash(local_url), 1)
    sleep(2)
    coinbase_id = mine_block(local_url, get_addresshash(local_url), 1)
    sleep(2)
    coinbase_id = mine_block(local_url, get_addresshash(local_url), 1)
    sleep(2)
    coinbase_id = mine_block(local_url, get_addresshash(local_url), 1)
    #print_all_balance(all_url)
