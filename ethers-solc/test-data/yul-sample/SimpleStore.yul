object "SimpleStore" {
    code {
      datacopy(0, dataoffset("Runtime"), datasize("Runtime"))
      return(0, datasize("Runtime"))
    }
    object "Runtime" {
      code {
  function mslice(position, length) -> result {
    result := div(mload(position), exp(2, sub(256, mul(length, 8))))
  }
  function StoreCalldata.sig(pos) -> res {
    res := mslice(StoreCalldata.sig.position(pos), 4)
  }
  function StoreCalldata.sig.position(_pos) -> _offset {
          function StoreCalldata.sig.position._chunk0(pos) -> __r {
            __r := 0x00
          }
          function StoreCalldata.sig.position._chunk1(pos) -> __r {
            __r := pos
          }
        _offset := add(StoreCalldata.sig.position._chunk0(_pos), add(StoreCalldata.sig.position._chunk1(_pos), 0))

  }
  function StoreCalldata.val(pos) -> res {
    res := mslice(StoreCalldata.val.position(pos), 32)
  }
  function StoreCalldata.val.position(_pos) -> _offset {
         function StoreCalldata.val.position._chunk0(pos) -> __r {
            __r := 0x04
          }

          function StoreCalldata.val.position._chunk1(pos) -> __r {
            __r := pos
          }


        _offset := add(StoreCalldata.val.position._chunk0(_pos), add(StoreCalldata.val.position._chunk1(_pos), 0))
  }
        calldatacopy(0, 0, 36) // write calldata to memory
        switch StoreCalldata.sig(0) // select signature from memory (at position 0)

        case 0x6057361d { // new signature method
          sstore(0, StoreCalldata.val(0)) // sstore calldata value
          log2(0, 0, 0x69404ebde4a368ae324ed310becfefc3edfe9e5ebca74464e37ffffd8309a3c1, StoreCalldata.val(0))
        }

        case 0x6d4ce63c {
          mstore(100, sload(0))
          return (100, 32)
        }
      }
    }
  }
