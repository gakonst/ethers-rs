pragma solidity >=0.8.4;

contract SimpleRevertingStorage {
    event ValueChanged(
        address indexed author,
        address indexed oldAuthor,
        string oldValue,
        string newValue
    );

    address public lastSender;
    string _value;
    string _otherValue;

    constructor(string memory value) {
        emit ValueChanged(msg.sender, address(0), _value, value);
        _value = value;
    }

    function getValue(bool rev) external view returns (string memory) {
        require(!rev, "getValue revert");
        return _value;
    }

    function setValue(string memory value, bool rev) external {
        require(!rev, "setValue revert");
        emit ValueChanged(msg.sender, lastSender, _value, value);
        _value = value;
        lastSender = msg.sender;
    }

    event Deposit(uint256 value);

    function deposit() external payable {
        emit Deposit(msg.value);
    }

    function emptyRevert() external pure {
        revert();
    }

    function stringRevert(string calldata data) external pure {
        revert(data);
    }

    error CustomError();

    function customError() external pure {
        revert CustomError();
    }

    error CustomErrorWithData(string);

    function customErrorWithData(string calldata data) external pure {
        revert CustomErrorWithData(data);
    }
}
