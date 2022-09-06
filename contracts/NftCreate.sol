// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./INFT.sol";

contract NftSale {

	INFT public nft;

	//@todo change it
	//pk - 5aefce0a2d473f59578fa7dee6a122d6509af1e0f79fcbee700dfcfeddabe4cc
	address public verifyAddress = 0x4604F4045bb2b2d998dEd660081eb6ebC19C9f1e;
    address payable public ethReceiver;

	constructor(
		address _nftAddress,
        address _ethReceiver
	) {
		nft = INFT(_nftAddress);
        ethReceiver = payable(_ethReceiver);
	}

	function buyWithSignature(bytes memory _signature, uint tokenID) public payable {
		require(verify(_signature, tokenID, msg.value), "invalid signature");
		nft.mint(msg.sender, tokenID);
		_safeTransferETH(ethReceiver, address(this).balance);
	}

	function mintWithSignature(bytes memory _signature, uint tokenID) public {
		require(verify(_signature, tokenID), "invalid signature");
		nft.mint(msg.sender, tokenID);
	}

	/// signature methods.
	function verify(bytes memory _signature, uint tokenID) internal view returns(bool) {
		bytes32 message = prefixed(keccak256(abi.encodePacked(msg.sender, address(this), tokenID)));
        return (recoverSigner(message, _signature) == verifyAddress);
	}

    function verify(bytes memory _signature, uint tokenID, uint value) internal view returns(bool) {
		bytes32 message = prefixed(keccak256(abi.encodePacked(msg.sender, address(this), tokenID, value)));
        return (recoverSigner(message, _signature) == verifyAddress);
	}

    function recoverSigner(bytes32 message, bytes memory sig)
        internal
        pure
        returns (address)
    {
        (uint8 v, bytes32 r, bytes32 s) = abi.decode(sig, (uint8, bytes32, bytes32));

        return ecrecover(message, v, r, s);
    }

    /// builds a prefixed hash to mimic the behavior of eth_sign.
    function prefixed(bytes32 hash) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked("\x19Ethereum Signed Message:\n32", hash));
    }

	// internal functions
	function _safeTransferETH(address payable to, uint256 value) internal 
	{
		(bool success, ) = to.call{value: value}('');
		require(success, 'ETH_TRANSFER_FAILED');
    }
}
