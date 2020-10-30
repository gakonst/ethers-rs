r#"[{
  "output": "0x",
  "stateDiff": {
	"0x5df9b87991262f6ba471f09758cde1c0fc1de734": {
	  "balance": {
		"+": "0x7a69"
	  },
	  "code": {
		"+": "0x"
	  },
	  "nonce": {
		"+": "0x0"
	  },
	  "storage": {}
	},
	"0xa1e4380a3b1f749673e270229993ee55f35663b4": {
	  "balance": {
		"*": {
		  "from": "0x6c6b935b8bbd400000",
		  "to": "0x6c5d01021be7168597"
		}
	  },
	  "code": "=",
	  "nonce": {
		"*": {
		  "from": "0x0",
		  "to": "0x1"
		}
	  },
	  "storage": {}
	},
	"0xe6a7a1d47ff21b6321162aea7c6cb457d5476bca": {
	  "balance": {
		"*": {
		  "from": "0xf3426785a8ab466000",
		  "to": "0xf350f9df18816f6000"
		}
	  },
	  "code": "=",
	  "nonce": "=",
	  "storage": {}
	}
  },
  "trace": [
	{
	  "action": {
		"callType": "call",
		"from": "0xa1e4380a3b1f749673e270229993ee55f35663b4",
		"gas": "0x0",
		"input": "0x",
		"to": "0x5df9b87991262f6ba471f09758cde1c0fc1de734",
		"value": "0x7a69"
	  },
	  "result": {
		"gasUsed": "0x0",
		"output": "0x"
	  },
	  "subtraces": 0,
	  "traceAddress": [],
	  "type": "call"
	}
  ],
  "transactionHash": "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060",
  "vmTrace": {
	"code": "0x",
	"ops": []
  }
}]"#
