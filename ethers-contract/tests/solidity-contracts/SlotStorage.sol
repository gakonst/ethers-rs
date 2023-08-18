pragma solidity >=0.4.24;

contract SlotStorage {
    event ValueChanged(
        address indexed author,
        address indexed oldAuthor,
        bytes32 oldValue,
        bytes32 newValue
    );

    bytes32 private constant KEY =
        bytes32(
            0xa35a6bd95953594c6d23a75dc715af91915e970ba4d87f1141e13b915e0201a3
        );

    address public lastSender;

    constructor(bytes32 value) {
        bytes32 _value = getValue();
        emit ValueChanged(msg.sender, address(0), _value, value);
        setValue(value);
    }

    function getValue() public view returns (bytes32 val) {
        val = readBytes32(KEY);
    }

    function setValue(bytes32 value) public returns (bytes32 val) {
        bytes32 _value = getValue();
        emit ValueChanged(msg.sender, lastSender, _value, value);
        writeBytes32(KEY, value);
        lastSender = msg.sender;
        val = _value;
    }

    function writeBytes32(bytes32 _key, bytes32 _val) internal {
        assembly {
            sstore(_key, _val)
        }
    }

    function readBytes32(bytes32 _key) internal view returns (bytes32 val) {
        assembly {
            val := sload(_key)
        }
    }
}
