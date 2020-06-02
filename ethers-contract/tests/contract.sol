pragma solidity >=0.4.24;

contract SimpleStorage {

    event ValueChanged(address indexed author, address indexed oldAuthor, string oldValue, string newValue);

    address public lastSender;
    string _value;

    constructor(string memory value) public {
        emit ValueChanged(msg.sender, address(0), _value, value);
        _value = value;
    }

    function getValue() view public returns (string memory) {
        return _value;
    }

    function setValue(string memory value) public {
        emit ValueChanged(msg.sender, lastSender, _value, value);
        _value = value;
        lastSender = msg.sender;
    }
}
