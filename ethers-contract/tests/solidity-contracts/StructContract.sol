// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.10;

contract MyContract {
    struct Point {
        uint256 x;
        uint256 y;
    }

    event NewPoint(Point x);

    function submitPoint(Point memory _point) public {
        emit NewPoint(_point);
    }
}