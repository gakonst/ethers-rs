// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.10;

/// ============ External Imports ============

import "../libs/base64-sol/base64.sol";
import "../libs/openzeppelin/MerkleProof.sol";
import "../libs/solmate/ERC721.sol";

/// ============ Internal Imports ============

import "./LilOwnable.sol";

/// ============ Defaults ============

library Defaults {
    string internal constant DefaultDescription =
        "Globally Recognized Avatars on the Ethereum Blockchain";
    string internal constant DefaultForDefaultImage = "robohash";
}

/// @title ProtoGravaNFT
/// @notice Gravatar-powered ERC721 claimable by members of a Merkle tree
/// @author Davis Shaver <davisshaver@gmail.com>
contract ProtoGravaNFT is ERC721, LilOwnable {
    uint256 public constant TOTAL_SUPPLY = type(uint256).max - 1;
    uint256 public totalSupply;

    /// ============ Events ============

    /// @notice Emitted after a successful mint
    /// @param to which address
    /// @param hash that was claimed
    /// @param name that was used
    event Mint(address indexed to, string hash, string name);

    /// ============ Errors ============

    /// @notice Thrown if a non-existent token is queried
    error DoesNotExist();
    /// @notice Thrown if unauthorized user tries to burn token
    error NotAuthorized();
    /// @notice Thrown if total supply is exceeded
    error NoTokensLeft();
    /// @notice Thrown if address/hash are not part of Merkle tree
    error NotInMerkle();

    /// ============ Mutable Storage ============

    /// @notice Mapping of ids to hashes
    mapping(uint256 => string) private gravIDsToHashes;

    /// @notice Mapping of ids to names
    mapping(uint256 => string) private gravIDsToNames;

    /// @notice Mapping of addresses to hashes
    mapping(address => string) private gravOwnersToHashes;

    /// @notice Merkle root
    bytes32 public merkleRoot;

    /// @notice Default fallback image
    string public defaultFormat;

    /// @notice Description
    string public description;

    /// ============ Constructor ============

    /// @notice Creates a new ProtoGravaNFT contract
    /// @param _name of token
    /// @param _symbol of token
    /// @param _merkleRoot of claimees
    constructor(
        string memory _name,
        string memory _symbol,
        bytes32 _merkleRoot
    ) ERC721(_name, _symbol) {
        defaultFormat = Defaults.DefaultForDefaultImage;
        description = Defaults.DefaultDescription;
        merkleRoot = _merkleRoot;
    }

    /* solhint-disable quotes */
    /// @notice Generates a Gravatar image URI for token
    /// @param gravatarHash for this specific token
    /// @param name for this specific token
    /// @return Token URI
    function formatTokenURI(string memory gravatarHash, string memory name)
        public
        view
        returns (string memory)
    {
        return
            string(
                abi.encodePacked(
                    "data:application/json;base64,",
                    Base64.encode(
                        abi.encodePacked(
                            bytes(
                                abi.encodePacked(
                                    '{"name": "',
                                    name,
                                    '", "description": "',
                                    description,
                                    '", "image": "//secure.gravatar.com/avatar/',
                                    gravatarHash,
                                    "?s=2048&d=",
                                    defaultFormat,
                                    '"}'
                                )
                            )
                        )
                    )
                )
            );
    }

    /* solhint-enable quotes */

    /// @notice Mint a token
    /// @param name of token being minted
    /// @param gravatarHash of token being minted
    /// @param proof of Gravatar hash ownership
    function mint(
        string calldata name,
        string calldata gravatarHash,
        bytes32[] calldata proof
    ) external {
        if (totalSupply + 1 >= TOTAL_SUPPLY) revert NoTokensLeft();

        bytes32 leaf = keccak256(abi.encodePacked(gravatarHash, msg.sender));
        bool isValidLeaf = MerkleProof.verify(proof, merkleRoot, leaf);
        if (!isValidLeaf) revert NotInMerkle();

        uint256 newItemId = totalSupply++;
        gravIDsToHashes[newItemId] = gravatarHash;
        gravIDsToNames[newItemId] = name;

        _mint(msg.sender, newItemId);

        emit Mint(msg.sender, gravatarHash, name);
    }

    /// @notice Gets URI for a specific token
    /// @param id of token being queried
    /// @return Token URI
    function tokenURI(uint256 id) public view override returns (string memory) {
        if (ownerOf[id] == address(0)) revert DoesNotExist();

        return formatTokenURI(gravIDsToHashes[id], gravIDsToNames[id]);
    }

    /// @notice Update default Gravatar image format for future tokens
    /// @param _defaultFormat for Gravatar image API
    function ownerSetDefaultFormat(string calldata _defaultFormat) public {
        if (msg.sender != _owner) revert NotOwner();
        defaultFormat = _defaultFormat;
    }

    /// @notice Update default Gravatar image format for future tokens
    /// @param _description for tokens
    function ownerSetDescription(string calldata _description) public {
        if (msg.sender != _owner) revert NotOwner();
        description = _description;
    }

    /// @notice Set a new Merkle root
    /// @param _merkleRoot for validating claims
    function ownerSetMerkleRoot(bytes32 _merkleRoot) public {
        if (msg.sender != _owner) revert NotOwner();
        merkleRoot = _merkleRoot;
    }

    /// @notice Get the description
    /// @return Description
    function getDescription() public view returns (string memory) {
        return description;
    }

    /// @notice Get the default image format
    /// @return Default image format
    function getDefaultImageFormat() public view returns (string memory) {
        return defaultFormat;
    }

    /// @notice Declare supported interfaces
    /// @param interfaceId for support check
    /// @return Boolean for interface support
    function supportsInterface(bytes4 interfaceId)
        public
        pure
        override(LilOwnable, ERC721)
        returns (bool)
    {
        return
            interfaceId == 0x7f5828d0 || // ERC165 Interface ID for ERC173
            interfaceId == 0x80ac58cd || // ERC165 Interface ID for ERC721
            interfaceId == 0x5b5e139f || // ERC165 Interface ID for ERC165
            interfaceId == 0x01ffc9a7; // ERC165 Interface ID for ERC721Metadata
    }
}
