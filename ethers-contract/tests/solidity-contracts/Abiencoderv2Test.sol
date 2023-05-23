pragma solidity >=0.6.0;
pragma experimental ABIEncoderV2;

// note that this file is not synced with Abiencoderv2Test.json
contract AbiencoderV2Test {
    struct Person {
        string name;
        uint age;
    }
    function defaultPerson() public pure returns (Person memory) {
        return Person("Alice", 20);
    }
}
