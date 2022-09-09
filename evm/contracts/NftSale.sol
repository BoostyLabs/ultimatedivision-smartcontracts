// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./INFT.sol";

contract NftSale {

	uint public constant MAX_UNITS_PER_ADDRESS = 11;
	uint public constant INITIAL_PRICE = 0.08 ether;
	// @todo - change start time
	uint public constant START_TIME = 1635717600;
	
	address payable public ethReceiver;
	INFT public nft;

	//@todo change it
	//pk - 5aefce0a2d473f59578fa7dee6a122d6509af1e0f79fcbee700dfcfeddabe4cc
	address public verifyAddress = 0x4604F4045bb2b2d998dEd660081eb6ebC19C9f1e;

	mapping(address => bool) public presaleBuyers;
	mapping(address => uint) public buyers;

	constructor(
		address _nftAddress,
		address payable _ethReceiver
	) {
		nft = INFT(_nftAddress);
		ethReceiver = _ethReceiver;
	}

	function buyBatch(uint amount) public payable {
		require(block.timestamp >= START_TIME, "sale is not started yet");
		require(buyers[msg.sender] + amount <= MAX_UNITS_PER_ADDRESS, "exceed MAX_UNITS_PER_ADDRESS");

		uint currentPrice = INITIAL_PRICE * amount;
		require(msg.value == currentPrice, "invalid value");
		
		buyers[msg.sender] += amount;

		nft.mintBatch(msg.sender, amount);
		ethReceiver.transfer(address(this).balance);
	}

	function buy() public payable {
		require(block.timestamp >= START_TIME, "sale is not started yet");
		require(buyers[msg.sender] < MAX_UNITS_PER_ADDRESS, "exceed MAX_UNITS_PER_ADDRESS");

		uint currentPrice = INITIAL_PRICE;
		require(msg.value == currentPrice, "invalid value");

		buyers[msg.sender]++;
		nft.mint(msg.sender);

		ethReceiver.transfer(address(this).balance);
	}

	function presaleBuyWithSignature(bytes memory _signature) public payable {
		require(!presaleBuyers[msg.sender], "only one token can be bought on presale");
		require(verify(_signature), "invalid signature");
		
		uint currentPrice = INITIAL_PRICE;
		require(msg.value == currentPrice, "invalid value");
		
		presaleBuyers[msg.sender] = true;

		nft.mint(msg.sender);

		ethReceiver.transfer(address(this).balance);
	}

	/// signature methods.
	function verify(bytes memory _signature) internal view returns(bool) {
		bytes32 message = prefixed(keccak256(abi.encodePacked(msg.sender, address(this))));
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
}
