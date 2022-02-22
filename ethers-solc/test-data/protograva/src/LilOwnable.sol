// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.10;

/// @title LilOwnable
/// @notice Ownable contract drop-in
abstract contract LilOwnable {
    /// ============ Mutable Storage ============

    /// @notice Contract owner
    address internal _owner;

    /// ============ Errors ============

    /// @notice Thrown if non-owner attempts to transfer
    error NotOwner();

    /// ============ Events ============

    /// @notice Emitted after ownership is transferred
    /// @param previousOwner of contract
    /// @param newOwner of contract
    event OwnershipTransferred(
        address indexed previousOwner,
        address indexed newOwner
    );

    /// ============ Constructor ============

    /// @notice Makes contract ownable
    constructor() {
        _owner = msg.sender;
    }

    /// @notice Owner of contract
    function owner() external view returns (address) {
        return _owner;
    }

    /// @notice Transfer ownership of contract
    /// @param _newOwner of contract
    function transferOwnership(address _newOwner) external {
        if (msg.sender != _owner) revert NotOwner();

        _owner = _newOwner;
    }

    /// @notice Renounce ownership of contract
    function renounceOwnership() public {
        if (msg.sender != _owner) revert NotOwner();

        _owner = address(0);
    }

    /// @notice Declare supported interfaces
    /// @param interfaceId for support check
    function supportsInterface(bytes4 interfaceId)
        public
        pure
        virtual
        returns (bool)
    {
        return interfaceId == 0x7f5828d0; // ERC165 Interface ID for ERC173
    }
}
