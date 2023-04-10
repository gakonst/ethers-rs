// SPDX-License-Identifier: MIT
pragma solidity ^0.8.4;

import "./@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract ERC20Example is ERC20 {
    constructor() ERC20("ERC20Example", "XYZ") {
        _mint(msg.sender, 1000 * 10 ** decimals());
    }
}
