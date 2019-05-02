#!/bin/bash
import requests
import json
import sys 

headers = {"content-type": "application/json; charset=UTF-8"}
session = requests.Session()
session.trust_env = False

def send_json(url, payload):
    response = session.post(url, headers=headers, data=json.dumps(payload), verify=False)
    response_native = json.loads(response.text)
    return response_native

if __name__ == "__main__":
    if len(sys.argv)<3:
        print("send_json miss args: url payload")
    else:    
        send_json(sys.argv[1], sys.argv[2])
	
