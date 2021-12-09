pragma solidity >=0.6.0;
pragma experimental ABIEncoderV2;

contract AbiencoderV2Test {
    struct Person {
        string name;
        uint age;
    }
    function defaultPerson() public pure returns (Person memory) {
        return Person("Alice", 20);
    }
}