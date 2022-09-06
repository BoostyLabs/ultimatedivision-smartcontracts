// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface INFT {
	function mint(address _to, uint _tokenId) external;
	function mintBatch(address _to, uint[] memory _tokenIds) external;
	function mintBatch(address _to, uint amount) external;
	function mint(address _to) external;
}
