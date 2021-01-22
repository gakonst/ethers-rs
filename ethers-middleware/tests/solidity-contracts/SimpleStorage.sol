pragma solidity >=0.4.24;

contract SimpleStorage {

    event ValueChanged(address indexed author, address indexed oldAuthor, uint256 oldValue, uint256 newValue);

    address public lastSender;
    uint256 public value;

    function setValue(uint256 _value) public {
        emit ValueChanged(msg.sender, lastSender, value, _value);
        value = _value;
        lastSender = msg.sender;
    }
}
