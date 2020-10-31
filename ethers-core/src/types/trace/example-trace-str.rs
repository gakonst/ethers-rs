r#"{
  "output": "0x",
  "stateDiff": {
    "0x01f0eb5c4b0a9d8285b67195f5f10ce22971a102": {
      "balance": {
        "*": {
          "from": "0x7361af5818297800",
          "to": "0x734a36bb22448000"
        }
      },
      "code": "=",
      "nonce": {
        "*": {
          "from": "0x1d6",
          "to": "0x1d7"
        }
      },
      "storage": {}
    },
    "0xb2930b35844a230f00e51431acae96fe543a0347": {
      "balance": {
        "*": {
          "from": "0x11b39d46046d14d44e5",
          "to": "0x11b39d687ebea8b3ce5"
        }
      },
      "code": "=",
      "nonce": "=",
      "storage": {}
    },
    "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3": {
      "balance": {
        "*": {
          "from": "0x109397d7f6f000",
          "to": "0x25e48fb49df000"
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
        "from": "0x01f0eb5c4b0a9d8285b67195f5f10ce22971a102",
        "gas": "0xa5f8",
        "input": "0x1a695230000000000000000000000000c227a75b32ed37d3f9d6341b9904d003dad3b1b3",
        "to": "0x0b95993a39a363d99280ac950f5e4536ab5c5566",
        "value": "0x1550f7dca70000"
      },
      "result": {
        "gasUsed": "0x1ddf",
        "output": "0x"
      },
      "subtraces": 1,
      "traceAddress": [],
      "type": "call"
    },
    {
      "action": {
        "callType": "call",
        "from": "0x0b95993a39a363d99280ac950f5e4536ab5c5566",
        "gas": "0x8fc",
        "input": "0x",
        "to": "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3",
        "value": "0x1550f7dca70000"
      },
      "result": {
        "gasUsed": "0x0",
        "output": "0x"
      },
      "subtraces": 0,
      "traceAddress": [
        0
      ],
      "type": "call"
    }
  ],
  "vmTrace": {
    "code": "0x60606040523615610055576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff1680631a6952301461005e5780637362377b1461008c5780638da5cb5b146100a1575b61005c5b5b565b005b61008a600480803573ffffffffffffffffffffffffffffffffffffffff169060200190919050506100f6565b005b341561009757600080fd5b61009f61013a565b005b34156100ac57600080fd5b6100b4610210565b604051808273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200191505060405180910390f35b8073ffffffffffffffffffffffffffffffffffffffff166108fc349081150290604051600060405180830381858888f19350505050151561013657600080fd5b5b50565b6000809054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff1614151561019557600080fd5b6000809054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166108fc3073ffffffffffffffffffffffffffffffffffffffff16319081150290604051600060405180830381858888f19350505050151561020d57600080fd5b5b565b6000809054906101000a900473ffffffffffffffffffffffffffffffffffffffff16815600a165627a7a7230582029eabe8a624d811f3ea09c310d65be79ddefa23e3b702541dc1687b475f091690029",
    "ops": [
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x60"
          ],
          "store": null,
          "used": 42485
        },
        "pc": 0,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x40"
          ],
          "store": null,
          "used": 42482
        },
        "pc": 2,
        "sub": null
      },
      {
        "cost": 12,
        "ex": {
          "mem": {
            "data": "0x0000000000000000000000000000000000000000000000000000000000000060",
            "off": 64
          },
          "push": [],
          "store": null,
          "used": 42470
        },
        "pc": 4,
        "sub": null
      },
      {
        "cost": 2,
        "ex": {
          "mem": null,
          "push": [
            "0x24"
          ],
          "store": null,
          "used": 42468
        },
        "pc": 5,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x0"
          ],
          "store": null,
          "used": 42465
        },
        "pc": 6,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x55"
          ],
          "store": null,
          "used": 42462
        },
        "pc": 7,
        "sub": null
      },
      {
        "cost": 10,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 42452
        },
        "pc": 10,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x0"
          ],
          "store": null,
          "used": 42449
        },
        "pc": 11,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x1a695230000000000000000000000000c227a75b32ed37d3f9d6341b9904d003"
          ],
          "store": null,
          "used": 42446
        },
        "pc": 13,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x100000000000000000000000000000000000000000000000000000000"
          ],
          "store": null,
          "used": 42443
        },
        "pc": 14,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x100000000000000000000000000000000000000000000000000000000",
            "0x1a695230000000000000000000000000c227a75b32ed37d3f9d6341b9904d003"
          ],
          "store": null,
          "used": 42440
        },
        "pc": 44,
        "sub": null
      },
      {
        "cost": 5,
        "ex": {
          "mem": null,
          "push": [
            "0x1a695230"
          ],
          "store": null,
          "used": 42435
        },
        "pc": 45,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0xffffffff"
          ],
          "store": null,
          "used": 42432
        },
        "pc": 46,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x1a695230"
          ],
          "store": null,
          "used": 42429
        },
        "pc": 51,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x1a695230",
            "0x1a695230"
          ],
          "store": null,
          "used": 42426
        },
        "pc": 52,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x1a695230"
          ],
          "store": null,
          "used": 42423
        },
        "pc": 53,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x1"
          ],
          "store": null,
          "used": 42420
        },
        "pc": 58,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x5e"
          ],
          "store": null,
          "used": 42417
        },
        "pc": 59,
        "sub": null
      },
      {
        "cost": 10,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 42407
        },
        "pc": 62,
        "sub": null
      },
      {
        "cost": 1,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 42406
        },
        "pc": 94,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x8a"
          ],
          "store": null,
          "used": 42403
        },
        "pc": 95,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x4"
          ],
          "store": null,
          "used": 42400
        },
        "pc": 98,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x4",
            "0x4"
          ],
          "store": null,
          "used": 42397
        },
        "pc": 100,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x4",
            "0x4"
          ],
          "store": null,
          "used": 42394
        },
        "pc": 101,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3"
          ],
          "store": null,
          "used": 42391
        },
        "pc": 102,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0xffffffffffffffffffffffffffffffffffffffff"
          ],
          "store": null,
          "used": 42388
        },
        "pc": 103,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3"
          ],
          "store": null,
          "used": 42385
        },
        "pc": 124,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3",
            "0x4"
          ],
          "store": null,
          "used": 42382
        },
        "pc": 125,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x20"
          ],
          "store": null,
          "used": 42379
        },
        "pc": 126,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x24"
          ],
          "store": null,
          "used": 42376
        },
        "pc": 128,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x24",
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3"
          ],
          "store": null,
          "used": 42373
        },
        "pc": 129,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3",
            "0x24",
            "0x4"
          ],
          "store": null,
          "used": 42370
        },
        "pc": 130,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x4",
            "0x24"
          ],
          "store": null,
          "used": 42367
        },
        "pc": 131,
        "sub": null
      },
      {
        "cost": 2,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 42365
        },
        "pc": 132,
        "sub": null
      },
      {
        "cost": 2,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 42363
        },
        "pc": 133,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0xf6"
          ],
          "store": null,
          "used": 42360
        },
        "pc": 134,
        "sub": null
      },
      {
        "cost": 8,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 42352
        },
        "pc": 137,
        "sub": null
      },
      {
        "cost": 1,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 42351
        },
        "pc": 246,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3",
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3"
          ],
          "store": null,
          "used": 42348
        },
        "pc": 247,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0xffffffffffffffffffffffffffffffffffffffff"
          ],
          "store": null,
          "used": 42345
        },
        "pc": 248,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3"
          ],
          "store": null,
          "used": 42342
        },
        "pc": 269,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x8fc"
          ],
          "store": null,
          "used": 42339
        },
        "pc": 270,
        "sub": null
      },
      {
        "cost": 2,
        "ex": {
          "mem": null,
          "push": [
            "0x1550f7dca70000"
          ],
          "store": null,
          "used": 42337
        },
        "pc": 273,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x1550f7dca70000",
            "0x8fc"
          ],
          "store": null,
          "used": 42334
        },
        "pc": 274,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x1550f7dca70000",
            "0x8fc",
            "0x1550f7dca70000"
          ],
          "store": null,
          "used": 42331
        },
        "pc": 275,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x0"
          ],
          "store": null,
          "used": 42328
        },
        "pc": 276,
        "sub": null
      },
      {
        "cost": 5,
        "ex": {
          "mem": null,
          "push": [
            "0x0"
          ],
          "store": null,
          "used": 42323
        },
        "pc": 277,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x0",
            "0x1550f7dca70000"
          ],
          "store": null,
          "used": 42320
        },
        "pc": 278,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x40"
          ],
          "store": null,
          "used": 42317
        },
        "pc": 279,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": {
            "data": "0x0000000000000000000000000000000000000000000000000000000000000060",
            "off": 64
          },
          "push": [
            "0x60"
          ],
          "store": null,
          "used": 42314
        },
        "pc": 281,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x0"
          ],
          "store": null,
          "used": 42311
        },
        "pc": 282,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x40"
          ],
          "store": null,
          "used": 42308
        },
        "pc": 284,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": {
            "data": "0x0000000000000000000000000000000000000000000000000000000000000060",
            "off": 64
          },
          "push": [
            "0x60"
          ],
          "store": null,
          "used": 42305
        },
        "pc": 286,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x60",
            "0x60"
          ],
          "store": null,
          "used": 42302
        },
        "pc": 287,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x60",
            "0x0",
            "0x60",
            "0x60",
            "0x60"
          ],
          "store": null,
          "used": 42299
        },
        "pc": 288,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x0"
          ],
          "store": null,
          "used": 42296
        },
        "pc": 289,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x60",
            "0x0",
            "0x60"
          ],
          "store": null,
          "used": 42293
        },
        "pc": 290,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x1550f7dca70000",
            "0x60",
            "0x0",
            "0x60",
            "0x0",
            "0x60",
            "0x1550f7dca70000"
          ],
          "store": null,
          "used": 42290
        },
        "pc": 291,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3",
            "0x0",
            "0x1550f7dca70000",
            "0x60",
            "0x0",
            "0x60",
            "0x0",
            "0x60",
            "0x1550f7dca70000",
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3"
          ],
          "store": null,
          "used": 42287
        },
        "pc": 292,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x0",
            "0x1550f7dca70000",
            "0x60",
            "0x0",
            "0x60",
            "0x0",
            "0x60",
            "0x1550f7dca70000",
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3",
            "0x0"
          ],
          "store": null,
          "used": 42284
        },
        "pc": 293,
        "sub": null
      },
      {
        "cost": 9700,
        "ex": {
          "mem": null,
          "push": [
            "0x1"
          ],
          "store": null,
          "used": 34884
        },
        "pc": 294,
        "sub": {
          "code": "0x",
          "ops": []
        }
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x1",
            "0x0",
            "0x1550f7dca70000",
            "0x60",
            "0xc227a75b32ed37d3f9d6341b9904d003dad3b1b3"
          ],
          "store": null,
          "used": 34881
        },
        "pc": 295,
        "sub": null
      },
      {
        "cost": 2,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 34879
        },
        "pc": 296,
        "sub": null
      },
      {
        "cost": 2,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 34877
        },
        "pc": 297,
        "sub": null
      },
      {
        "cost": 2,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 34875
        },
        "pc": 298,
        "sub": null
      },
      {
        "cost": 2,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 34873
        },
        "pc": 299,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x0"
          ],
          "store": null,
          "used": 34870
        },
        "pc": 300,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x1"
          ],
          "store": null,
          "used": 34867
        },
        "pc": 301,
        "sub": null
      },
      {
        "cost": 3,
        "ex": {
          "mem": null,
          "push": [
            "0x136"
          ],
          "store": null,
          "used": 34864
        },
        "pc": 302,
        "sub": null
      },
      {
        "cost": 10,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 34854
        },
        "pc": 305,
        "sub": null
      },
      {
        "cost": 1,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 34853
        },
        "pc": 310,
        "sub": null
      },
      {
        "cost": 1,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 34852
        },
        "pc": 311,
        "sub": null
      },
      {
        "cost": 2,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 34850
        },
        "pc": 312,
        "sub": null
      },
      {
        "cost": 8,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 34842
        },
        "pc": 313,
        "sub": null
      },
      {
        "cost": 1,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 34841
        },
        "pc": 138,
        "sub": null
      },
      {
        "cost": 0,
        "ex": {
          "mem": null,
          "push": [],
          "store": null,
          "used": 34841
        },
        "pc": 139,
        "sub": null
      }
    ]
  }
}"#
